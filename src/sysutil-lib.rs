#![allow(non_snake_case)]
#![allow(dead_code)]

use std::fs;
use std::fs::read_dir;
use std::io::Read;
use std::path;
use std::thread;
use std::path::Path;
use std::time::Duration;

fn readFile(filePath: &str) -> String {
    match fs::File::open(path::Path::new(filePath)) {
        Err(_) => {
            return String::new();
        },
        Ok(mut file) => {
            let mut buffer = String::new();

            match file.read_to_string(&mut buffer) {
                Err(_) => {
                    return String::new();
                },
                Ok(_) => {
                    return buffer.trim().to_string();
                }
            }
        }
    };
}

pub fn gpuUsage() -> f32 {
    let fileContent = readFile("/sys/class/drm/card0/device/gpu_busy_percent");
    return fileContent.parse::<f32>().unwrap();
}

#[derive(Debug)]
pub struct CpuUsage {
    average: ProcessorUsage,
    processors: Vec::<ProcessorUsage>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct ProcessorUsage {
    total: f32,
    user: f32,
    nice: f32,
    system: f32,
    idle: f32,
    iowait: f32,
    interrupt: f32,
    soft_interrupt: f32
}

impl ProcessorUsage {
    fn clone(&self) -> ProcessorUsage {
        return ProcessorUsage {
            total: self.total,
            user: self.user,
            nice: self.nice,
            system: self.system,
            idle: self.idle,
            iowait: self.iowait,
            interrupt: self.interrupt,
            soft_interrupt: self.soft_interrupt
        }
    }
}

fn getStats() -> Vec<Vec<usize>> {
    let fileContent = readFile("/proc/stat");

    let lines = fileContent.split("\n");
    let mut strLines = Vec::<String>::new();

    for currentLine in lines {
        if currentLine.find("cpu") != None {
            strLines.push(currentLine.to_string());
        }
    }

    let mut uLines = Vec::<Vec<usize>>::new();
    for line in strLines {
        let splittedLine = line.split(" ").into_iter();
        let mut fixedLine = Vec::<usize>::new();

        for chunk in splittedLine {
            if !chunk.is_empty() && chunk.find("cpu") == None {
                fixedLine.push(chunk.parse().unwrap());
            }
        }
        uLines.push(fixedLine);
    }

    return uLines;
}

pub fn cpuUsage() -> CpuUsage {
    let before = getStats();
    thread::sleep(Duration::from_millis(250));
    let after = getStats();

    let mut processors = Vec::<ProcessorUsage>::new();
    for i in 0..before.len() {

        let beforeLine = &before[i];
        let beforeSum = {
            let mut sum = 0;

            for element in beforeLine {
                sum += element;
            }
            sum
        };

        let afterLine = &after[i];
        let afterSum = {
            let mut sum = 0;

            for element in afterLine {
                sum += element;
            }
            sum
        };

        let delta: f32 = (afterSum - beforeSum) as f32;

        processors.push(
            ProcessorUsage{
                total: {
                    100_f32 - (afterLine[3] - beforeLine[3]) as f32 * 100_f32 / delta
                },
                user: {
                    (afterLine[0] - beforeLine[0]) as f32 * 100_f32 / delta
                },
                nice: {
                    (afterLine[1] - beforeLine[1]) as f32 * 100_f32 / delta
                },
                system: {
                    (afterLine[2] - beforeLine[2]) as f32 * 100_f32 / delta
                },
                idle: {
                    (afterLine[3] - beforeLine[3]) as f32 * 100_f32 / delta
                },
                iowait: {
                    (afterLine[4] - beforeLine[4]) as f32 * 100_f32 / delta
                },
                interrupt: {
                    (afterLine[5] - beforeLine[5]) as f32 * 100_f32 / delta
                },
                soft_interrupt: {
                    (afterLine[6] - beforeLine[6]) as f32 * 100_f32 / delta
                }
            }
        );
    }

    /*;
    */
    return CpuUsage {
        average: processors[0].clone(),
        processors: processors[1..].to_vec()
    };
}

pub fn cpuFrequency() -> f32 {
    let fileContent = readFile("/proc/cpuinfo");
    let mut frequencies: f32 = 0.0;
    let mut count = 0;

    for line in fileContent.split("\n") {
        if line.find("cpu MHz") != None {
            frequencies += line.split(" ").last().unwrap().parse::<f32>().unwrap();
            count += 1;
        }
    }

    return frequencies / (count as f32);

}

pub fn ramUsage() -> f32 {
    let content = readFile("/proc/meminfo");

    let mut memTotal = "";
    let mut memAvailable = "";

    for element in content.split("\n") {
        if element.find("MemTotal") != None {
            memTotal = element;
        } else if element.find("MemAvailable") != None {
            memAvailable = element;
        }
    }

    let uMemTotal = {
        let mut total = 0_usize;
        for element in memTotal.split(" ") {
            if element != "MemTotal:" && !element.is_empty() {
                total = element.parse::<usize>().unwrap();
                break
            }
        }
        total
    };

    let uMemAvailable = {
        let mut available = 0_usize;
        for element in memAvailable.split(" ") {
            if element != "MemAvailable:" && !element.is_empty() {
                available = element.parse::<usize>().unwrap();
                break
            }
        }
        available
    };

    return 100_f32 - uMemAvailable as f32 * 100_f32 / uMemTotal as f32;
}

fn getRate() -> (usize, usize) {
    let stats = readFile("/proc/net/dev");

    let mut downloadRate = 0_usize;
    let mut uploadRate = 0_usize;

    for line in stats.split("\n") {
        if line.find(":") != None {

            let splitted = {
                let tmp = line.split(" ");

                let mut data = Vec::<usize>::new();
                for chunk in tmp {
                    if !chunk.is_empty() && chunk.find(":") == None {
                        data.push(chunk.parse().unwrap());
                    }
                }
                data
            };

            downloadRate += splitted[0];
            uploadRate += splitted[8];

        }
    }
    return (downloadRate, uploadRate);
}

#[derive(Debug)]
pub struct NetworkRate {
    download: f32,
    upload: f32
}

pub fn networkRate() -> NetworkRate {
    let (downBefore, upBefore) = getRate();
    thread::sleep(Duration::from_millis(500));
    let (downAfter, upAfter) = getRate();

    let downloadRate: f32 = ((downAfter - downBefore) as f32) / 0.5_f32;
    let uploadRate: f32 = ((upAfter - upBefore) as f32) / 0.5_f32;

    return NetworkRate {
        download: downloadRate,
        upload: uploadRate
    };
}

#[derive(Debug)]
pub struct TemperatureSensor {
    label: String,
    temperature: f32
}

pub fn temperatureSensors() -> Vec<TemperatureSensor> {
    let hwmonPath = Path::new("/sys/class/hwmon");
    let dirs = fs::read_dir(hwmonPath).unwrap();

    let mut sensors = Vec::<TemperatureSensor>::new();

    for dir in dirs {
        let dirPath = dir.unwrap().path();

        let labelFile = dirPath.join("name");
        let label = readFile(labelFile.to_str().unwrap());

        let temperatureFile = dirPath.join("temp1_input");
        let temperature = readFile(temperatureFile.to_str().unwrap());

        sensors.push(
            TemperatureSensor {
                label: label,
                temperature: temperature.parse::<f32>().unwrap() / 1000_f32
            }
        );
    }
    return sensors;
}

#[derive(Debug)]
pub struct Cpu {
    modelName: String,
    cores: usize,
    threads: usize,
    dies: usize,
    governors: Vec<String>,
    maxFrequencyMHz: f32,
    architecture: String
}

pub fn cpuInfo() -> Cpu  {
    let infoFile = readFile("/proc/cpuinfo");
    let modelName = {
        let mut name = String::new();
        for line in infoFile.split("\n") {
            if line.contains("model name") {
                name = line.split(":").last().unwrap().to_string();
                break
            }
        }
        name.trim().to_string()
    };

    let baseDir = Path::new("/sys/devices/system/cpu");

    let mut coreCount: usize = 0;
    let mut dieCount: usize = 0;

    for processor in read_dir(baseDir).unwrap() {

        let processorPath = processor.unwrap().path();
        let path = processorPath.to_str().unwrap();

        if path.find("cpu") != None &&
           path.find("cpufreq") == None &&
           path.find("cpuidle") == None {

            let coreId = readFile(format!("{path}/topology/core_id").as_str());
            let dieId = readFile(format!("{path}/topology/die_id").as_str());

            if !coreId.is_empty() {
                let cid = coreId.parse::<usize>().unwrap();

                if cid > coreCount {
                    coreCount = cid;
                }
            }

            if !dieId.is_empty() {
                let did = dieId.parse::<usize>().unwrap();

                if did > dieCount {
                    dieCount = did;
                }
            }
        }
    }

    coreCount += 1;
    dieCount += 1;

    let cpuInfoFile = readFile("/proc/cpuinfo");
    let threadCount = cpuInfoFile.matches("processor").count();

    let mut governors = Vec::<String>::new();
    let policiesPath = Path::new("/sys/devices/system/cpu/cpufreq/");

    let mut maxFrequency: usize = 0;

    for dir in read_dir(policiesPath).unwrap() {
        let path = dir.unwrap().path();
        let sPath = path.to_str().unwrap();

        if sPath.contains("policy") {
            let localGovernors = readFile(format!("{sPath}/scaling_available_governors").as_str());
            let maxFreq = readFile(format!("{sPath}/cpuinfo_max_freq").as_str());

            if !maxFreq.is_empty() {
                match maxFreq.parse::<usize>(){
                    Err(_) => (),
                    Ok(freq) => {
                        if freq > maxFrequency {
                            maxFrequency = freq;
                        }
                    }
                }
            }

            for governor in localGovernors.split(" ") {
                if !governors.contains(&governor.to_string())  {
                    governors.push(governor.to_string());
                }
            }
        }
    }

    let freqMHz = maxFrequency as f32 / 1000_f32;
    let maxInteger: usize = usize::MAX;

    let arch = {
        if maxInteger as u128 == 2_u128.pow(64) - 1 {
            String::from("64 bit")

        } else if maxInteger as u128 == 2_u128.pow(32) - 1 {
            String::from("32 bit")

        } else {
            String::new()
        }
    };

    return Cpu{
        modelName: modelName,
        cores: coreCount,
        threads: threadCount,
        dies: dieCount,
        governors: governors,
        maxFrequencyMHz: freqMHz,
        architecture: arch
    };
}

#[derive(Debug)]
pub struct RamSize {
    gb: f32,
    gib: f32
}

pub fn ramSize() -> RamSize {
    let content = readFile("/proc/meminfo");

    let mut memTotal = "";

    for element in content.split("\n") {
        if element.find("MemTotal") != None {
            memTotal = element;
        }
    }

    let uMemTotal = {
        let mut total = 0_usize;
        for element in memTotal.split(" ") {
            if element != "MemTotal:" && !element.is_empty() {
                total = element.parse::<usize>().unwrap();
                break
            }
        }
        total
    };
    let GiB = uMemTotal as f32 * 1000_f32 / 1024_f32 / 1024_f32 / 1024_f32;
    let GB = uMemTotal as f32 / 1000_f32 / 1000_f32;

    return RamSize {
        gb: GB,
        gib: GiB
    };
}

#[derive(Debug)]
pub struct SchedulerPolicy {
    name: String,
    scalingGovernor: String,
    scalingDriver: String,
    minimumScalingMHz: f32,
    maximumScalingMHz: f32
}

pub fn schedulerInfo() -> Vec<SchedulerPolicy> {
    let schedulerDir = path::Path::new("/sys/devices/system/cpu/cpufreq/");
    let mut policies = Vec::<SchedulerPolicy>::new();

    for dir in read_dir(schedulerDir).unwrap() {
        let path = dir.unwrap().path();
        let sPath = path.to_str().unwrap();

        if sPath.contains("policy") {
            let policyName = sPath.split("/").last().unwrap().to_string();

            let scalingGovernor = readFile(format!("{sPath}/scaling_governor").as_str());
            let scalingDriver = readFile(format!("{sPath}/scaling_driver").as_str());

            let maxScalingFrequency = readFile(
                format!("{sPath}/scaling_max_freq").as_str()
            ).parse::<f32>().unwrap() / 1000_f32;

            let minScalingFrequency = readFile(
                format!("{sPath}/scaling_min_freq").as_str()
            ).parse::<f32>().unwrap() / 1000_f32;

            policies.push(
                SchedulerPolicy{
                    name: policyName,
                    scalingGovernor: scalingGovernor,
                    scalingDriver: scalingDriver,
                    minimumScalingMHz: minScalingFrequency,
                    maximumScalingMHz: maxScalingFrequency
                }
            );
        }
    }

    return policies;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        println!("{:?}",cpuUsage());
        println!("RAM usage: {:?}", ramUsage());

        println!("{:?}", networkRate());
        println!("GPU usage: {:?}", gpuUsage());

        println!("{:?}", temperatureSensors());
        println!("{:?}", cpuInfo());

        println!("{:?}", ramSize());
        println!("{:?}", schedulerInfo());

        assert_eq!(String::new(), String::new());
    }
}