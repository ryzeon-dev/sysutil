//! Linux system information library
//!
//! <div class="warning">This library is ment to be used only in linux systems. It is possible to write code using it on other systems, but it will not allow to run the code, panicking before execution </div>
//!
//! # Installation
//!
//! Run the following command in your terminal
//!
//! ```bash
//! cargo add sysutil
//! ```
//!
//! # Importation
//! Add in your code:
//! ```
//! use sysutil;
//! ```
//!
//! ---
//! <div class="warning">GPU's related functions have been tested only on AMD Radeon 7000 series, any other GPU model is not "officially supported"</div>

#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub mod gpu;
pub mod cpu;
pub mod ram;
pub mod network;
pub mod storage;
pub mod motherboard;
pub mod sensors;
pub mod bus;
mod utils;
pub use utils::{ByteSize, ByteUnit};

use rsjson::{Json, Node, NodeContent};

/// Returns a `rsjson::Json` object containing all the data which `sysutil` can extract
pub fn exportJson() -> rsjson::Json {
    let mut json = rsjson::Json::new();

    let mut cpuNodeContent = rsjson::Json::new();
    let cpu = cpu::CPU::new();

    cpuNodeContent.addNode(rsjson::Node::new(
        "model-name".to_string(),
        NodeContent::String(cpu.info.modelName)
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "cores".to_string(),
        NodeContent::Int(cpu.info.cores)
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "threads".to_string(),
        NodeContent::Int(cpu.info.threads)
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "dies".to_string(),
        NodeContent::Int(cpu.info.dies)
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "governors".to_string(),
        NodeContent::List(Vec::<NodeContent>::from({
            let mut list = Vec::<NodeContent>::new();

            for governor in cpu.info.governors {
                list.push(NodeContent::String(governor));
            }

            list
        }))
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "max-frequency".to_string(),
        NodeContent::Float(cpu.info.maxFrequencyMHz)
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "clock-boost".to_string(),
        {
            if cpu.info.clockBoost == None {
                NodeContent::Null
            } else {
                NodeContent::Bool(cpu.info.clockBoost.unwrap())
            }
        }
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "architecture".to_string(),
        NodeContent::String(cpu.info.architecture)
    ));

    cpuNodeContent.addNode(rsjson::Node::new(
        "byte-order".to_string(),
        NodeContent::String(cpu.info.byteOrder)
    ));

    let mut cpuAverageUsageNodeContent = Json::new();
    cpuAverageUsageNodeContent.addNode(Node::new(
        "total".to_string(),
        NodeContent::Float(cpu.averageUsage.total)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "user".to_string(),
        NodeContent::Float(cpu.averageUsage.user)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "nice".to_string(),
        NodeContent::Float(cpu.averageUsage.nice)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "system".to_string(),
        NodeContent::Float(cpu.averageUsage.system)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "idle".to_string(),
        NodeContent::Float(cpu.averageUsage.idle)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "iowait".to_string(),
        NodeContent::Float(cpu.averageUsage.iowait)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "interrupt".to_string(),
        NodeContent::Float(cpu.averageUsage.interrupt)
    ));

    cpuAverageUsageNodeContent.addNode(Node::new(
        "soft-interrupt".to_string(),
        NodeContent::Float(cpu.averageUsage.soft_interrupt)
    ));

    let mut cpuSchedulerPolicyContent = Json::new();

    for policy in cpu.schedulerPolicies {
        let mut policyNodeContent = Json::new();

        policyNodeContent.addNode(Node::new(
            "scaling-governor".to_string(),
            NodeContent::String(policy.scalingGovernor)
        ));

        policyNodeContent.addNode(Node::new(
            "scaling-driver".to_string(),
            NodeContent::String(policy.scalingDriver)
        ));

        policyNodeContent.addNode(Node::new(
            "minimum-scaling-mhz".to_string(),
            NodeContent::Float(policy.minimumScalingMHz)
        ));

        policyNodeContent.addNode(Node::new(
            "maximum-scaling-mhz".to_string(),
            NodeContent::Float(policy.maximumScalingMHz)
        ));

        cpuSchedulerPolicyContent.addNode(Node::new(
            policy.name.to_string(),
            NodeContent::Json(policyNodeContent)
        ));
    }

    cpuNodeContent.addNode(Node::new(
        "usage".to_string(),
        NodeContent::Json(cpuAverageUsageNodeContent)
    ));

    cpuNodeContent.addNode(Node::new(
        "scheduler-policies".to_string(),
        NodeContent::Json(cpuSchedulerPolicyContent)
    ));

    cpuNodeContent.addNode(Node::new(
        "frequency".to_string(),
        NodeContent::Int(cpu.averageFrequency.khz)
    ));

    let mut cpuClockSourceNodeContent = Json::new();
    let clockSource = cpu::clockSource();

    cpuClockSourceNodeContent.addNode(Node::new(
        "current".to_string(),
        NodeContent::String(clockSource.current)
    ));

    cpuClockSourceNodeContent.addNode(Node::new(
        "available".to_string(),
        NodeContent::List({
            let mut list = Vec::<NodeContent>::new();

            for source in clockSource.available {
                list.push(NodeContent::String(source));
            }

            list
        })
    ));

    cpuNodeContent.addNode(Node::new(
        "clock-source".to_string(),
        NodeContent::Json(cpuClockSourceNodeContent)
    ));

    json.addNode(rsjson::Node::new(
        "cpu".to_string(),
        NodeContent::Json(cpuNodeContent)
    ));

    let mut ramNodeContent = Json::new();
    let ram = ram::RAM::new();

    ramNodeContent.addNode(Node::new(
        "usage".to_string(),
        NodeContent::Float(ram.usage)
    ));

    ramNodeContent.addNode(Node::new(
        "size-gib".to_string(),
        NodeContent::Float(ram.size.GiB())
    ));

    ramNodeContent.addNode(Node::new(
        "frequency",
        match ram.frequency {
            Some(frequency) => {
                NodeContent::Int(frequency)
            },
            None => NodeContent::Null
        }
    ));

    ramNodeContent.addNode(Node::new(
        "width",
        match ram.busWidth {
            Some(width) => {
                NodeContent::Int(width)
            },
            None => NodeContent::Null
        }
    ));

    json.addNode(Node::new(
        "ram".to_string(),
        NodeContent::Json(ramNodeContent)
    ));

    let mut motherBoardNodeContent = Json::new();
    let motherboard = motherboard::motherboardInfo();

    motherBoardNodeContent.addNode(Node::new(
        "name".to_string(),
        NodeContent::String(motherboard.name)
    ));

    motherBoardNodeContent.addNode(Node::new(
        "vendor".to_string(),
        NodeContent::String(motherboard.vendor)
    ));

    motherBoardNodeContent.addNode(Node::new(
        "version".to_string(),
        NodeContent::String(motherboard.version)
    ));

    let mut biosNodeContent = Json::new();
    biosNodeContent.addNode(Node::new(
        "vendor".to_string(),
        NodeContent::String(motherboard.bios.vendor)
    ));

    biosNodeContent.addNode(Node::new(
        "release".to_string(),
        NodeContent::String(motherboard.bios.release)
    ));

    biosNodeContent.addNode(Node::new(
        "version".to_string(),
        NodeContent::String(motherboard.bios.version)
    ));

    biosNodeContent.addNode(Node::new(
        "date".to_string(),
        NodeContent::String(motherboard.bios.date)
    ));

    motherBoardNodeContent.addNode(Node::new(
        "bios".to_string(),
        NodeContent::Json(biosNodeContent)
    ));

    json.addNode(Node::new(
        "motherboard".to_string(),
        NodeContent::Json(motherBoardNodeContent)
    ));

    let mut nvmeDevicesNodeContent = Vec::<NodeContent>::new();
    let nvmeDevices = storage::nvmeDevices();

    for device in nvmeDevices {
        let mut deviceNodeContent = Json::new();

        deviceNodeContent.addNode(Node::new(
            "device".to_string(),
            NodeContent::String(device.device)
        ));

        deviceNodeContent.addNode(Node::new(
            "pcie-address".to_string(),
            NodeContent::String(device.pcieAddress)
        ));

        deviceNodeContent.addNode(Node::new(
            "model".to_string(),
            NodeContent::String(device.model)
        ));

        deviceNodeContent.addNode(Node::new(
            "link-speed-gts".to_string(),
            NodeContent::Float(device.linkSpeedGTs)
        ));

        deviceNodeContent.addNode(Node::new(
            "pcie-lanes".to_string(),
            NodeContent::Int(device.pcieLanes)
        ));

        deviceNodeContent.addNode(Node::new(
            "size".to_string(),
            NodeContent::Int(device.size.b())
        ));

        let mut partitionsNodeList = Vec::<NodeContent>::new();
        for partition in device.partitions {
            let mut partitionNodeContent = Json::new();

            partitionNodeContent.addNode(Node::new(
                "device".to_string(),
                NodeContent::String(partition.device)
            ));

            partitionNodeContent.addNode(Node::new(
                "mount-point".to_string(),
                NodeContent::String(partition.mountPoint)
            ));

            partitionNodeContent.addNode(Node::new(
                "filesystem".to_string(),
                NodeContent::String(partition.fileSystem)
            ));

            partitionNodeContent.addNode(Node::new(
                "size".to_string(),
                NodeContent::Int(partition.size.b())
            ));

            partitionNodeContent.addNode(Node::new(
                "start-point".to_string(),
                NodeContent::Int(partition.startPoint)
            ));

            partitionsNodeList.push(NodeContent::Json(partitionNodeContent));
        }

        deviceNodeContent.addNode(Node::new(
            "partitions".to_string(),
            NodeContent::List(partitionsNodeList)
        ));

        nvmeDevicesNodeContent.push(NodeContent::Json(deviceNodeContent));
    }

    json.addNode(Node::new(
        "nvme-devices".to_string(),
        NodeContent::List(nvmeDevicesNodeContent)
    ));

    let mut storageDevicesNodeContent = Vec::<NodeContent>::new();
    let storageDevices = storage::storageDevices();

    for device in storageDevices {
        let mut deviceNodeContent = Json::new();

        deviceNodeContent.addNode(Node::new(
            "device".to_string(),
            NodeContent::String(device.device)
        ));

        deviceNodeContent.addNode(Node::new(
            "model".to_string(),
            NodeContent::String(device.model)
        ));


        deviceNodeContent.addNode(Node::new(
            "size".to_string(),
            NodeContent::Int(device.size.b())
        ));

        let mut partitionsNodeList = Vec::<NodeContent>::new();
        for partition in device.partitions {
            let mut partitionNodeContent = Json::new();

            partitionNodeContent.addNode(Node::new(
                "device".to_string(),
                NodeContent::String(partition.device)
            ));

            partitionNodeContent.addNode(Node::new(
                "mount-point".to_string(),
                NodeContent::String(partition.mountPoint)
            ));

            partitionNodeContent.addNode(Node::new(
                "filesystem".to_string(),
                NodeContent::String(partition.fileSystem)
            ));

            partitionNodeContent.addNode(Node::new(
                "size".to_string(),
                NodeContent::Int(partition.size.b())
            ));

            partitionNodeContent.addNode(Node::new(
                "start-point".to_string(),
                NodeContent::Int(partition.startPoint)
            ));

            partitionsNodeList.push(NodeContent::Json(partitionNodeContent));
        }

        deviceNodeContent.addNode(Node::new(
            "partitions".to_string(),
            NodeContent::List(partitionsNodeList)
        ));

        storageDevicesNodeContent.push(NodeContent::Json(deviceNodeContent));
    }

    json.addNode(Node::new(
        "storage-devices".to_string(),
        NodeContent::List(storageDevicesNodeContent)
    ));

    match sensors::batteryInfo() {
        None => {
            json.addNode(Node::new(String::from("battery"), NodeContent::Null));
        },
        Some(battery) => {
            let mut batteryNodeContent = Json::new();

            batteryNodeContent.addNode(Node::new(
                String::from("capacity"),
                NodeContent::Int(battery.capacity as usize)
            ));

            batteryNodeContent.addNode(Node::new(
                String::from("status"),
                match battery.status {
                    sensors::BatteryStatus::Charging => NodeContent::String(String::from("Charging")),
                    sensors::BatteryStatus::Discharging => NodeContent::String(String::from("Discharging")),
                    sensors::BatteryStatus::Full => NodeContent::String(String::from("Full")),
                }
            ));

            json.addNode(Node::new(
                String::from("battery"),
                NodeContent::Json(batteryNodeContent)
            ));
        }
    };

    match sensors::getBacklight() {
        None => {
            json.addNode(Node::new(String::from("backlight"), NodeContent::Null));
        },
        Some(backlight) => {
            let mut brightnessNodeContent = Json::new();

            brightnessNodeContent.addNode(Node::new(
                String::from("brightness"),
                NodeContent::Int(backlight.brightness as usize)
            ));

            brightnessNodeContent.addNode(Node::new(
                String::from("max-brightness"),
                NodeContent::Int(backlight.maxBrightness as usize)
            ));

            json.addNode(Node::new(
                String::from("backlight"),
                NodeContent::Json(brightnessNodeContent)
            ));
        }
    }

    let mut networkNodeContent = rsjson::Json::new();
    let mut networkRateNodeContent = rsjson::Json::new();

    let networkRate = network::networkRate();

    networkRateNodeContent.addNode(Node::new(
        "download",
        NodeContent::Float(networkRate.download)
    ));

    networkRateNodeContent.addNode(Node::new(
        "upload",
        NodeContent::Float(networkRate.upload)
    ));

    networkNodeContent.addNode(Node::new(
        "rate",
        NodeContent::Json(networkRateNodeContent)
    ));

    let mut networkRoutesNodeConent = Vec::<NodeContent>::new();
    let routes = network::networkRoutes();

    for route in routes {
        let mut routeNodeContent = rsjson::Json::new();

        routeNodeContent.addNode(Node::new(
            "type",
            NodeContent::String(match route.routeType {
                network::RouteType::TCP => String::from("TCP"),
                network::RouteType::TCP6 => String::from("TCP6"),
                network::RouteType::UDP => String::from("UDP"),
                network::RouteType::UDP6 => String::from("UDP6")
            })
        ));

        routeNodeContent.addNode(Node::new(
            "local-address",
            NodeContent::String(route.localAddress)
        ));

        routeNodeContent.addNode(Node::new(
            "local-port",
            NodeContent::Int(route.localPort as usize)
        ));

        routeNodeContent.addNode(Node::new(
            "remote-address",
            NodeContent::String(route.remoteAddress)
        ));

        routeNodeContent.addNode(Node::new(
            "remote-port",
            NodeContent::Int(route.remotePort as usize)
        ));

        routeNodeContent.addNode(Node::new(
            "route-status",
            NodeContent::String(route.routeStatus.toString())
        ));

        networkRoutesNodeConent.push(NodeContent::Json(routeNodeContent));
    }

    networkNodeContent.addNode(Node::new(
        "routes",
        NodeContent::List(networkRoutesNodeConent)
    ));

    json.addNode(Node::new(
        "network",
        NodeContent::Json(networkNodeContent)
    ));

    let temperatureSensors = sensors::temperatureSensors();
    let mut temperatureSensorsNodeContent = Vec::<NodeContent>::new();

    for sensor in temperatureSensors {
        let mut temperatureSensorNodeContent = Json::new();

        temperatureSensorNodeContent.addNode(Node::new(
            "label",
            NodeContent::String(sensor.label)
        ));

        temperatureSensorNodeContent.addNode(Node::new(
            "temperature",
            match sensor.temperature {
                None => NodeContent::Null,
                Some(temp) => NodeContent::Float(temp)
            }
        ));

        temperatureSensorsNodeContent.push(NodeContent::Json(temperatureSensorNodeContent));
    }

    json.addNode(Node::new(
        "temperature-sensors",
        NodeContent::List(temperatureSensorsNodeContent)
    ));

    let mut vramNodeContent = Json::new();
    let vram = gpu::VRAM::new();

    vramNodeContent.addNode(Node::new(
        "size-gib",
        match vram.size {
            Some(size) => {
                NodeContent::Float(size.GiB())
            },

            None => {
                NodeContent::Null
            }
        }
    ));

    vramNodeContent.addNode(Node::new(
        "usage",
        match vram.usage {
            Some(usage) => {
                NodeContent::Float(usage)
            },

            None => {
                NodeContent::Null
            }
        }
    ));

    vramNodeContent.addNode(Node::new(
        "frequency",
        match vram.frequency {
            Some(frequency) => {
                NodeContent::Int(frequency)
            },

            None => {
                NodeContent::Null
            }
        }
    ));

    vramNodeContent.addNode(Node::new(
        "bus-width",
        match vram.busWidth {
            Some(width) => {
                NodeContent::Int(width)
            },

            None => {
                NodeContent::Null
            }
        }
    ));

    json.addNode(Node::new(
        "vram",
        NodeContent::Json(vramNodeContent)
    ));

    match gpu::gpuMetrics() {
        None => {
            json.addNode(Node::new(
                "gpu-metrics",
                NodeContent::Null
            ));
        },
        Some(metrics) => {
            let mut metricsNodeContent = Json::new();

            metricsNodeContent.addNode(Node::new(
                "temperature-edge",
                NodeContent::Int(metrics.temperatureEdge as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "temperature-hotspot",
                NodeContent::Int(metrics.temperatureHotspot as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "temperature-mem",
                NodeContent::Int(metrics.temperatureMem as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "temperature-vrgfx",
                NodeContent::Int(metrics.temperatureVrgfx as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "temperature-vrsoc",
                NodeContent::Int(metrics.temperatureVrsoc as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "temperature-vrmem",
                NodeContent::Int(metrics.temperatureVrmem as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "average-socket-power",
                NodeContent::Int(metrics.averageSocketPower as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "average-gfxclk-frequency",
                NodeContent::Int(metrics.averageGfxclkFrequency as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "average-sockclk-frequency",
                NodeContent::Int(metrics.averageSockclkFrequency as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "average-uclk-frequency",
                NodeContent::Int(metrics.averageUclkFrequency as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "current-gfxclk",
                NodeContent::Int(metrics.currentGfxclk as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "current-sockclk",
                NodeContent::Int(metrics.currentSockclk as usize)
            ));

            metricsNodeContent.addNode(Node::new(
                "throttle-status",
                NodeContent::Int(metrics.throttleStatus as usize),
            ));

            metricsNodeContent.addNode(Node::new(
                "current-fan-speed",
                NodeContent::Int(metrics.currentFanSpeed as usize),
            ));

            metricsNodeContent.addNode(Node::new(
                "pcie-link-width",
                NodeContent::Int(metrics.pcieLinkWidth as usize),
            ));

            metricsNodeContent.addNode(Node::new(
                "pcie-link-speed",
                NodeContent::Int(metrics.pcieLinkSpeed as usize),
            ));

            json.addNode(Node::new(
                "gpu-metrics",
                NodeContent::Json(metricsNodeContent)
            ));
        }
    }

    let load = cpu::getLoad();

    let mut loadNodeContent = Json::new();
    loadNodeContent.addNode(Node::new(
        "one-minute", NodeContent::Float(load.oneMinute)
    ));

    loadNodeContent.addNode(Node::new(
        "five-minutes", NodeContent::Float(load.fiveMinutes)
    ));

    loadNodeContent.addNode(Node::new(
        "fifteen-minutes", NodeContent::Float(load.fifteenMinutes)
    ));

    json.addNode(Node::new(
        "load", NodeContent::Json(loadNodeContent)
    ));

    let mut ipv4NodeContent = Vec::<NodeContent>::new();
    for ipv4 in network::getIPv4() {
        let mut ipNode = rsjson::Json::new();

        ipNode.addNode(Node::new(
            "address",
            NodeContent::String(ipv4.address)
        ));

        ipNode.addNode(Node::new(
            "interface",
            NodeContent::String(ipv4.interface)
        ));

        ipv4NodeContent.push(NodeContent::Json(ipNode));
    }

    json.addNode(Node::new(
        "ipv4",
        NodeContent::List(ipv4NodeContent)
    ));

    let mut busInputNodeContent = Vec::<NodeContent>::new();
    for input in bus::busInput() {
        let mut inputNode = rsjson::Json::new();

        inputNode.addNode(Node::new(
            "bus",
            NodeContent::Int(input.bus as usize)
        ));

        inputNode.addNode(Node::new(
            "vendor",
            NodeContent::Int(input.vendor as usize)
        ));

        inputNode.addNode(Node::new(
            "product",
            NodeContent::Int(input.product as usize)
        ));

        inputNode.addNode(Node::new(
            "version",
            NodeContent::Int(input.version as usize)
        ));

        inputNode.addNode(Node::new(
            "physical-path",
            NodeContent::String(input.physicalPath)
        ));

        inputNode.addNode(Node::new(
            "sysfs-path",
            NodeContent::String(input.sysfsPath)
        ));

        inputNode.addNode(Node::new(
            "name",
            NodeContent::String(input.name)
        ));

        inputNode.addNode(Node::new(
            "handles",
            NodeContent::List({
                let mut binding = Vec::<NodeContent>::new();

                for handle in input.handles {
                    binding.push(NodeContent::String(handle));
                }

                binding
            })
        ));

        inputNode.addNode(Node::new(
            "properties",
            NodeContent::Int(input.properties as usize)
        ));

        inputNode.addNode(Node::new(
            "events",
            NodeContent::Int(input.events as usize)
        ));

        inputNode.addNode(Node::new(
            "keys",
            NodeContent::List({
                let mut binding = Vec::<NodeContent>::new();

                for key in input.keys {
                    binding.push(NodeContent::String(key));
                }

                binding
            })
        ));

        inputNode.addNode(Node::new(
            "miscellaneous-events",
            NodeContent::Int(input.miscellaneousEvents as usize)
        ));

        inputNode.addNode(Node::new(
            "led",
            NodeContent::Int(input.led as usize)
        ));

        busInputNodeContent.push(NodeContent::Json(inputNode));
    }

    json.addNode(Node::new(
        "bus-input",
        NodeContent::List(busInputNodeContent)
    ));

    let netIfaces = network::networkInterfaces();
    let mut ifacesNodeContent = rsjson::Json::new();

    for iface in netIfaces {
        let mut ifaceNodeContent = rsjson::Json::new();

        ifaceNodeContent.addNode(Node::new(
            "mac",
            NodeContent::String(iface.macAddress)
        ));

        ifaceNodeContent.addNode(Node::new(
            "interface-type",
            NodeContent::String(match iface.interfaceType {
                network::InterfaceType::Physical => String::from("physical"),
                network::InterfaceType::Virtual => String::from("virtual")
            })
        ));

        ifacesNodeContent.addNode(Node::new(
            iface.name,
            NodeContent::Json(ifaceNodeContent)
        ));
    }

    json.addNode(Node::new(
        "network-interfaces",
        NodeContent::Json(ifacesNodeContent)
    ));

    return json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // println!("{:?}", storage::nvmeDevices());

        /*println!("{:?}", cpuUsage());
        println!("RAM usage: {:?}", ramUsage());

        println!("{:?}", networkRate().download / 1024_f32 / 1024_f32);
        println!("GPU usage: {:?}", gpuUsage());

        println!("{:?}", temperatureSensors());
        println!("{:?}", cpuInfo());

        println!("{:?}", ramSize());
        println!("{:?}", schedulerInfo());

        println!("{:?}", batteryInfo());
        println!("{:?}", vramSize());

        println!("VRAM usage: {:?}", vramUsage());
        println!("{:?}", networkRoutes());

        let cpu = CPU::new();
        println!("{:?}", cpu);

        println!("{:?}", cpuFrequency());

        println!("{:?}", clockSource());
        println!("{:?}", motherboardInfo());

        println!("{:?}", gpuMetrics());
        println!("{:?}", nvmeDevices());
        println!("{:?}", storageDevices());

        println!("{:?}", getBacklight());
        println!("{:?}", getLoad());*/

        //println!("{:?}", getIPv4());
        /*println!("{:?}", busInput());*/

        /*let j = exportJson();
        j.writeToFile("file.json");*/

        // for i in cpu::cacheLevels() {
        //     let (level, size) = i;
        //
        //     let (size, unit) = size.fitBase1024();
        //     println!("{level} {} {}", size, unit.toString());
        // }

        println!("{:?}", network::networkInterfaces());

        assert_eq!(0_u8, 0_u8);
    }
}