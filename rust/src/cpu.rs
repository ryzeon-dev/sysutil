use std::{fs, path, thread};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use crate::utils::{*};
use regex;

/// Contains the average CPU usage and the discrete usage for each processor
#[derive(Debug, Clone)]
pub struct CpuUsage {
    pub average: ProcessorUsage,
    pub processors: Vec<ProcessorUsage>,
}

/// Encloses the different parameters relative to processor usage
#[derive(Debug, Clone)]
pub struct ProcessorUsage {
    pub total: f32,
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub idle: f32,
    pub iowait: f32,
    pub interrupt: f32,
    pub soft_interrupt: f32,
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
            soft_interrupt: self.soft_interrupt,
        };
    }
}

/// Contains base information relative to the CPU
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub modelName: String,
    pub cores: usize,
    pub threads: usize,
    pub dies: usize,
    pub governors: Vec<String>,
    pub maxFrequencyMHz: f32,
    pub clockBoost: Option<bool>,
    pub architecture: String,
    pub byteOrder: String
}

/// Encloses all CPU-related data available in the library
/// ## Example
/// Once generating a `CPU` instance, usages and scheduler policies can be updated by the `update()` method
/// ```rust
/// use sysutil;
///
/// let mut cpu = sysutil::cpu::CPU::new();
/// cpu.update();
/// ```
#[derive(Debug, Clone)]
pub struct CPU {
    pub info: CpuInfo,
    pub averageUsage: ProcessorUsage,
    pub perProcessorUsage: Vec<ProcessorUsage>,
    pub schedulerPolicies: Vec<SchedulerPolicy>,
    pub averageFrequency: Frequency,
    pub perProcessorFrequency: Vec<ProcessorFrequency>
}

impl CPU {
    pub fn new() -> CPU {
        let cpuUsage = cpuUsage();
        let frequency = cpuFrequency();

        CPU {
            info: cpuInfo(),
            averageUsage: cpuUsage.average,
            perProcessorUsage: cpuUsage.processors,
            schedulerPolicies: schedulerInfo(),
            averageFrequency: frequency.average,
            perProcessorFrequency: frequency.processors
        }
    }

    pub fn update(&mut self) {
        self.schedulerPolicies = schedulerInfo();
        let cpuUsage = cpuUsage();

        self.averageUsage = cpuUsage.average;
        self.perProcessorUsage = cpuUsage.processors;
    }
}

/// Frequency data structure implements direct conversion for frequencies
/// in various size orders
/// ```rust
/// use sysutil::cpu::cpuFrequency;
/// let frequency = cpuFrequency().average;
///
/// frequency.khz(); // returns the frequency in Kilo Hertz
/// frequency.mhz(); // returns the frequency in Mega Hertz
/// frequency.ghz(); // return the frequency in Giga Hertz
/// ```
#[derive(Debug, Clone)]
pub struct Frequency {
    pub khz: usize
}

impl Frequency {
    pub fn khz(&self) -> f32 {
        return self.khz as f32;
    }

    pub fn mhz(&self) -> f32 {
        return self.khz as f32 / 1000_f32;
    }

    pub fn ghz(&self) -> f32 {
        return self.khz as f32 / 1000_000_f32;
    }
}

/// Contains processor id and its frequency
#[derive(Debug, Clone)]
pub struct ProcessorFrequency {
    pub processorID: String,
    pub frequency: Frequency
}

/// Contains cpu frequencies, both average and processor wise
#[derive(Debug, Clone)]
pub struct CpuFrequency {
    pub average: Frequency,
    pub processors: Vec<ProcessorFrequency>
}

/// Contains currently active clock source and the available ones
#[derive(Debug, Clone)]
pub struct ClockSource {
    pub current: String,
    pub available: Vec<String>
}


/// Contains scheduler information relative to a processor in the system
#[derive(Debug, Clone)]
pub struct SchedulerPolicy {
    pub name: String,
    pub scalingGovernor: String,
    pub scalingDriver: String,
    pub minimumScalingMHz: f32,
    pub maximumScalingMHz: f32,
}

/// Holds data structure for average load
#[derive(Debug)]
pub struct Load {
    pub oneMinute: f32,
    pub fiveMinutes: f32,
    pub fifteenMinutes: f32
}

fn getStats() -> Vec<Vec<usize>> {
    linuxCheck();

    let fileContent = readFile("/proc/stat");

    let lines = fileContent.split("\n");
    let mut strLines = Vec::<String>::new();

    for currentLine in lines {
        if currentLine.contains("cpu") {
            strLines.push(currentLine.to_string());
        }
    }

    let mut uLines = Vec::<Vec<usize>>::new();
    for line in strLines {
        let splittedLine = line.split(' ');
        let mut fixedLine = Vec::<usize>::new();

        for chunk in splittedLine {
            if !chunk.is_empty() && !chunk.contains("cpu") {
                fixedLine.push(chunk.parse().unwrap());
            }
        }
        uLines.push(fixedLine);
    }

    return uLines;
}

/// Returns CPU usage, both average and processor-wise, each value is in percentage
pub fn cpuUsage() -> CpuUsage {
    linuxCheck();

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

        processors.push(ProcessorUsage {
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
            },
        });
    }

    return CpuUsage {
        average: processors[0].clone(),
        processors: processors[1..].to_vec(),
    };
}

/// Returns CPU base information, enclosed in the `CpuInfo` data structure

fn cpuInfo() -> CpuInfo {
    linuxCheck();

    let infoFile = readFile("/proc/cpuinfo");
    let modelName = {
        let mut name = String::new();
        for line in infoFile.split("\n") {
            if line.contains("model name") {
                name = line.split(":").last().unwrap().to_string();
                break;
            }
        }
        name.trim().to_string()
    };

    let baseDir = path::Path::new("/sys/devices/system/cpu");

    let mut coreCount: usize = 0;
    let mut dies = Vec::<usize>::new();

    for processor in fs::read_dir(baseDir).unwrap() {
        let processorPath = processor.unwrap().path();
        let path = processorPath.to_str().unwrap();

        if path.contains("cpu") && !path.contains("cpufreq") && !path.contains("cpuidle")
        {
            let coreId = readFile(format!("{path}/topology/core_id").as_str());
            let dieId = readFile(format!("{path}/topology/die_id").as_str());

            if !coreId.is_empty() {
                match coreId.parse::<usize>() {
                    Err(_) => (),
                    Ok(uCoreId) => {
                        if uCoreId > coreCount {
                            coreCount = uCoreId;
                        }
                    }
                }
            }

            if !dieId.is_empty() {
                match dieId.parse::<usize>() {
                    Err(_) => (),
                    Ok(uDieId) => {
                        if !dies.contains(&uDieId) {
                            dies.push(uDieId);
                        }
                    }
                }
            }
        }
    }

    let dieCount = dies.len();

    if coreCount % 2 == 1 {
        coreCount += 1;
    }

    let cpuInfoFile = readFile("/proc/cpuinfo");
    let threadCount = cpuInfoFile.matches("processor").count();

    let mut governors = Vec::<String>::new();
    let policiesPath = path::Path::new("/sys/devices/system/cpu/cpufreq/");

    let mut maxFrequency: usize = 0;
    let mut clockBoost: Option<bool> = None;

    for dir in fs::read_dir(policiesPath).unwrap() {
        let path = dir.unwrap().path();
        let sPath = path.to_str().unwrap();

        if sPath.contains("policy") {
            let localGovernors = readFile(format!("{sPath}/scaling_available_governors").as_str());
            let maxFreq = readFile(format!("{sPath}/cpuinfo_max_freq").as_str());

            if !maxFreq.is_empty() {
                match maxFreq.parse::<usize>() {
                    Err(_) => (),
                    Ok(freq) => {
                        if freq > maxFrequency {
                            maxFrequency = freq;
                        }
                    }
                }
            }

            for governor in localGovernors.split(" ") {
                if !governors.contains(&governor.to_string()) {
                    governors.push(governor.to_string());
                }
            }
        } else if sPath.contains("boost") {
            let content = readFile(sPath);

            if content.trim() == "1" {
                clockBoost = Some(true);
            } else {
                clockBoost = Some(false);
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

    let pipe = Command::new("sh").arg("-c").args(
        ["echo -n I | od -t o2 | head -n 1 | cut -f 2 -d \" \" | cut -c 6"]
    ).output().unwrap().stdout;

    let byteOrder = match String::from_utf8(pipe).unwrap().trim() {
        "1" => String::from("Little Endian"),
        "0" => String::from("Big Endian"),
        _ => String::new(),
    };

    return CpuInfo {
        modelName: modelName,
        cores: coreCount,
        threads: threadCount,
        dies: dieCount,
        governors: governors,
        maxFrequencyMHz: freqMHz,
        clockBoost: clockBoost,
        architecture: arch,
        byteOrder: byteOrder,
    };
}

/// Returns scheduler information for each processor
pub fn schedulerInfo() -> Vec<SchedulerPolicy> {
    linuxCheck();

    let schedulerDir = path::Path::new("/sys/devices/system/cpu/cpufreq/");
    let mut policies = Vec::<SchedulerPolicy>::new();

    for dir in fs::read_dir(schedulerDir).unwrap() {
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

            policies.push(SchedulerPolicy {
                name: policyName,
                scalingGovernor: scalingGovernor,
                scalingDriver: scalingDriver,
                minimumScalingMHz: minScalingFrequency,
                maximumScalingMHz: maxScalingFrequency,
            });
        }
    }

    return policies;
}

/// Returns the currently active clock source and the different ones available, enclosed in `ClockSource` struct
pub fn clockSource() -> ClockSource {
    linuxCheck();

    let currentClockSource = readFile(
        "/sys/devices/system/clocksource/clocksource0/current_clocksource"
    );

    let availableClockSources = readFile(
        "/sys/devices/system/clocksource/clocksource0/available_clocksource"
    );

    let mut sources = Vec::<String>::new();
    for source in availableClockSources.split(" ") {
        sources.push(String::from(source));
    }

    ClockSource {
        current: currentClockSource,
        available: sources,
    }
}

/// Returns cpu frequency, both average and processor wise
pub fn cpuFrequency() -> CpuFrequency {
    linuxCheck();
    let mut totalFreq: f32 = 0_f32;
    let mut frequencies: Vec<ProcessorFrequency> = Vec::new();

    let fileContent = readFile("/proc/cpuinfo");
    for chunk in fileContent.split("\n\n") {

        if chunk.is_empty() {
            continue
        }

        let mut id = String::new();
        let mut freq: f32 = 0_f32;

        for line in chunk.split("\n") {
            if line.contains("processor") {
                id = line.trim().split(":").last().unwrap().trim().to_string();

            } else if line.contains("cpu MHz") {
                freq = line.trim().split(":").last().unwrap().trim().parse::<f32>().unwrap();
            }
        }

        if id.is_empty() || freq == 0_f32 {
            continue
        }

        totalFreq += freq;
        frequencies.push(ProcessorFrequency{
            processorID: id,
            frequency: Frequency {
                khz: (freq * 1000.0) as usize
            }
        });
    }

    CpuFrequency {
        average: Frequency {
            khz: (totalFreq * 1000.0) as usize / frequencies.len()
        },
        processors: frequencies
    }
}

/// Returns the average load for the last one, five and fifteen minutes
pub fn getLoad() -> Load {
    let fileContent = readFile("/proc/loadavg");
    let binding = fileContent.split(" ").collect::<Vec<&str>>();

    Load {
        oneMinute: binding.get(0).unwrap().parse::<f32>().unwrap(),
        fiveMinutes: binding.get(1).unwrap().parse::<f32>().unwrap(),
        fifteenMinutes: binding.get(2).unwrap().parse::<f32>().unwrap()
    }
}

/// Returns a HashMap containing the size (in ByteSize) for each cache level
pub fn cacheLevels() -> HashMap<String, ByteSize> {
    let baseDir = "/sys/devices/system/cpu";
    let mut cpus = Vec::<String>::new();

    let pattern = regex::Regex::new(r"cpu[0-9]{1,3}").unwrap();

    for element in std::fs::read_dir(baseDir).unwrap() {
        let name = element.unwrap().file_name().to_str().unwrap().to_string();
        match pattern.captures(&name) {
            Some(_) => {
                cpus.push(name);
            },

            None => {

            }
        }
    }

    let mut cacheLevels = HashMap::<String, Vec<(String, String)>>::new();

    for cpu in cpus {
        let mut cacheDirs = Vec::<String>::new();

        let pattern = regex::Regex::new(r"index[0-9]{1}").unwrap();

        for element in std::fs::read_dir(format!("{}/{}/cache/", baseDir, cpu)).unwrap() {
            let name = element.unwrap().file_name().to_str().unwrap().to_string();
            match pattern.captures(&name) {
                Some(_) => {
                    cacheDirs.push(name);
                },
                None => {}
            }
        }

        for directory in cacheDirs {
            let level = readFile(format!("{}/{}/cache/{}/level", baseDir, cpu, directory));
            let size = readFile(format!("{}/{}/cache/{}/size", baseDir, cpu, directory));
            let shared = readFile(format!("{}/{}/cache/{}/shared_cpu_list", baseDir, cpu, directory));

            if !cacheLevels.contains_key(&level) {
                cacheLevels.insert(level, vec![(size, shared)]);

            } else {
                let mut savedCacheLevel = cacheLevels.get(&level).unwrap().clone();

                let mut found = false;

                for savedLevel in &savedCacheLevel {
                    let (savedSize, savedShared) = savedLevel;
                    if savedShared == &shared {
                        found = true;
                    }
                }

                if !found {
                    cacheLevels.remove(&level);
                    savedCacheLevel.push((size, shared));

                    cacheLevels.insert(level, savedCacheLevel);
                }
            }
        }
    }

    println!("{:?}", cacheLevels);

    let mut levels = HashMap::<String, ByteSize>::new();
    for level in cacheLevels.keys() {
        let mut totalSize: usize = 0;

        let chunks = cacheLevels.get(level);
        for chunk in chunks {
            let (size, _) = chunk.first().unwrap();

            let intSize = {
                if size.contains("K") {
                    size.replace("K", "").parse::<usize>().unwrap() * 1024

                } else if size.contains("M") {
                    size.replace("M", "").parse::<usize>().unwrap() * 1024
                } else {
                    size.parse::<usize>().unwrap()
                }
            };

            totalSize += intSize;
        }

        levels.insert(level.to_string(), ByteSize::fromBytes(totalSize));
    }

    return levels;
}