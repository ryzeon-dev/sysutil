use std::{fmt, fs, thread};
use std::time::Duration;
use crate::utils::{*};
/// Contains total download and upload newtwork rate (in bytes)
#[derive(Debug, Clone)]
pub struct NetworkRate {
    pub download: f32,
    pub upload: f32,
}

/// Different route types
#[derive(Debug, Clone, PartialEq)]
pub enum RouteType {
    TCP,
    TCP6,
    UDP,
    UDP6
}

#[derive(Debug, Clone)]
pub enum RouteStatus {
    ESTABLISHED,
    SYN_SENT,
    SYN_RECEIVED,
    FIN_WAIT1,
    FIN_WAIT2,
    TIME_WAIT,
    CLOSED,
    CLOSE_WAIT,
    LAST_ACKNOWLEDGMENT,
    LISTENING,
    CLOSING,
    NEW_SYN_RECEIVED
}

impl RouteStatus {
    fn fromTcpCode(code: &str) -> RouteStatus {
        if code == "01" {
            return RouteStatus::ESTABLISHED

        } else if code == "02" {
            return RouteStatus::SYN_SENT

        } else if code == "03" {
            return RouteStatus::SYN_RECEIVED

        } else if code == "04" {
            return RouteStatus::FIN_WAIT1

        } else if code == "05" {
            return RouteStatus::FIN_WAIT2

        } else if code == "06" {
            return RouteStatus::TIME_WAIT

        } else if code == "07" {
            return RouteStatus::CLOSED

        } else if code == "08" {
            return RouteStatus::CLOSE_WAIT

        } else if code == "09" {
            return RouteStatus::LAST_ACKNOWLEDGMENT

        } else if code == "0A" {
            return RouteStatus::LISTENING

        } else if code == "0B" {
            return RouteStatus::CLOSING

        } else {
            return RouteStatus::NEW_SYN_RECEIVED
        }
    }

    pub fn toString(&self) -> String {
        match self {
            RouteStatus::ESTABLISHED => String::from("established"),
            RouteStatus::SYN_SENT => String::from("syn sent"),
            RouteStatus::SYN_RECEIVED => String::from("syn received"),
            RouteStatus::FIN_WAIT1 => String::from("fin wait 1"),
            RouteStatus::FIN_WAIT2 => String::from("fin wait 2"),
            RouteStatus::TIME_WAIT => String::from("time wait"),
            RouteStatus::CLOSED => String::from("closed"),
            RouteStatus::CLOSE_WAIT => String::from("close wait"),
            RouteStatus::LAST_ACKNOWLEDGMENT => String::from("last acknowledgment"),
            RouteStatus::LISTENING => String::from("listening"),
            RouteStatus::CLOSING => String::from("closing"),
            RouteStatus::NEW_SYN_RECEIVED => String::from("new syn received")
        }
    }
}

/// Represents a network route and its type, containing local address+port, remote address+port and connection status
#[derive(Debug, Clone)]
pub struct NetworkRoute {
    pub routeType: RouteType,
    pub localAddress: String,
    pub localPort: u16,
    pub remoteAddress: String,
    pub remotePort: u16,
    pub routeStatus: RouteStatus
}

/// Holds information related to an IP address
#[derive(Debug)]
pub struct IPv4 {
    pub address: String,
    pub interface: String,
    pub broadcast: String,
    pub netmask: String,
    pub cidr: String
}

impl fmt::Display for IPv4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.cidr)
    }
}

impl IPv4 {
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.address, self.cidr)
    }
}

#[derive(Debug, Clone)]
pub enum InterfaceType {
    Physical, Virtual
}

/// Contains information about network interfaces
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub macAddress: String,
    pub interfaceType: InterfaceType
}

fn makeNetmask(ip: &String, broadcast: &String) -> String {
    let splittedIp = {
        let binding = ip.split(".").collect::<Vec<&str>>();
        let mut bytesIp = Vec::<u8>::new();

        for chunk in binding {
            bytesIp.push(chunk.parse::<u8>().unwrap());
        }

        bytesIp
    };

    let splittedBrd = {
        let binding = broadcast.split(".").collect::<Vec<&str>>();
        let mut bytesIp = Vec::<u8>::new();

        for chunk in binding {
            bytesIp.push(chunk.parse::<u8>().unwrap());
        }

        bytesIp
    };

    let mut mask = Vec::<u8>::new();
    for i in 0..4 {
        let ipByte = splittedIp.get(i).unwrap().to_owned();

        let brdByte = splittedBrd.get(i).unwrap().to_owned();
        mask.push(!ipByte | (ipByte & !brdByte));
    }

    format!(
        "{}.{}.{}.{}", mask.get(0).unwrap(), mask.get(1).unwrap(),
        mask.get(2).unwrap(), mask.get(3).unwrap()
    )
}

fn bitsToByte(bits: &mut Vec<u8>) -> u8{
    bits.reverse();
    let mut byte: u8 = 0;

    for i in 0..8 {
        byte += bits[i] * 2_i32.pow(i as u32) as u8;
    }
    return byte;
}

fn netmaskFromCidr(cidr: u8) -> String {
    let mut bits = Vec::<u8>::new();

    for i in 0..32 {
        if i < cidr {
            bits.push(1);
        } else {
            bits.push(0);
        }
    }

    let mut mask = Vec::<u8>::new();
    let mut i: usize = 0;

    while i < 32 {
        mask.push(bitsToByte(&mut bits.get_mut(i..i+8).unwrap().to_vec()));
        i += 8;
    }

    return format!("{}.{}.{}.{}", mask[0], mask[1], mask[2], mask[3]);

}

fn makeu8Vec(ip: &String) -> Vec<u8> {
    let splitted = ip.split(".");
    let mut uIP = Vec::<u8>::new();

    for octet in splitted {
        uIP.push(octet.parse::<u8>().unwrap())
    }

    return uIP
}

fn ipToBaseNetwork(ip: &String, mask: &String) -> String {
    let ipVec = makeu8Vec(ip);
    let maskVec = makeu8Vec(mask);

    let mut baseNetwork: Vec<String> = Vec::<String>::new();
    for i in 0..4 {
        baseNetwork.push((ipVec[i] & maskVec[i]).to_string());
    }

    return baseNetwork.join(".");
}

/// Returns the various ip addresses associated to the various network interfaces in the device
pub fn getIPv4() -> Vec<IPv4> {
    let mut ipv4Addresses = Vec::<IPv4>::new();
    let mut addresses = Vec::<(String, String, String, String)>::new();

    let routeFile = readFile("/proc/net/route");
    let fibTrie = readFile("/proc/net/fib_trie");

    let mut index: usize = 0;
    let lines = fibTrie.split("\n").collect::<Vec<&str>>();

    while index < lines.len() {
        let line = lines.get(index).unwrap().to_string();

        if !line.contains("link UNICAST") {
            index += 1;
            continue
        }

        let cidr = line.trim().get(1..3).unwrap().to_string();
        index += 1;
        let binding = lines.get(index).unwrap().to_string().replace("|--", "");

        if binding.contains("+") {
            index += 1;
            continue

        } else if binding.contains("link UNICAST") {
            continue
        }

        let address = binding.trim().to_string();

        while index < lines.len() && !lines.get(index).unwrap().contains("host LOCAL") {
            index += 1;
        }

        index += 1;
        let binding = lines.get(index).unwrap().replace("|--", "");

        let broadcast = binding.trim().to_string();
        let netmask = netmaskFromCidr(cidr.parse::<u8>().unwrap());

        addresses.push((address, broadcast, netmask, cidr));
        index += 1;
    }

    let mut usedIps = Vec::<String>::new();

    for line in routeFile.split("\n") {
        if line.is_empty() ||  line.contains("Gateway") {
            continue
        }

        let splittedLine = line.split("\t").collect::<Vec<&str>>();
        let device = splittedLine.first().unwrap().trim().to_string();
        let network = bytesToAddress(splittedLine.get(1).unwrap().trim().to_string(), ".");

        let mut ip = String::new();
        let mut brd = String::new();

        let mut mask = String::new();
        let mut cidr = String::new();

        for (address, broadcast, netmask, cidrMask) in &addresses {


            let baseAddress = ipToBaseNetwork(address, netmask);
            if &network == &baseAddress {
                if usedIps.contains(address) {
                    break
                }

                brd = broadcast.to_string();
                ip = address.to_string();

                mask = netmask.to_string();
                cidr = cidrMask.to_string();

                usedIps.push(ip.clone());
                break
            }
        }

        if !ip.is_empty() {
            ipv4Addresses.push(IPv4 {
                address: ip,
                interface: device,
                broadcast: brd,
                netmask: mask,
                cidr: cidr
            });
        }
    }

    return ipv4Addresses;
}

fn bytesToAddress(address: String, separator: &str) -> String {
    let mut chunks = Vec::<String>::new();

    let mut index: usize = 0;
    while index < address.len() {
        chunks.push(
            i64::from_str_radix(
                &address[index..index+2],
                16
            ).unwrap().to_string()
        );
        index += 2;
    }

    chunks.reverse();
    return chunks.join(separator);
}

fn bytesToPort(port: String) -> u16 {
    let (LSB, MSB) = port.split_at(2);
    (
        (
            i64::from_str_radix(MSB, 16).unwrap() as u16
        ) << 8
    ) + (
        i64::from_str_radix(LSB, 16).unwrap() as u16
    )
}

fn getRoutes(file: String, separator: &str, routeType: RouteType) -> Vec<NetworkRoute> {
    let mut routes = Vec::<NetworkRoute>::new();

    for line in file.split('\n') {
        if !line.contains(':') {
            continue;
        }

        let splittedLine: Vec<&str> = line.trim().split(" ").collect();
        let local: Vec<&str> = splittedLine[1].split(":").collect();

        let localAddress = bytesToAddress(local[0].to_string(), separator);
        let localPort = bytesToPort(local[1].to_string());

        let remote: Vec<&str> = splittedLine[2].split(":").collect();
        let remoteAddress = bytesToAddress(remote[0].to_string(), separator);
        let remotePort = bytesToPort(remote[1].to_string());

        let statusCode = splittedLine[3].trim();

        let status = {
            if routeType == RouteType::TCP || routeType == RouteType::TCP6 {
                RouteStatus::fromTcpCode(statusCode)

            } else {
                RouteStatus::LISTENING
            }
        };

        routes.push(
            NetworkRoute {
                routeType: routeType.clone(),
                localAddress: localAddress,
                localPort: localPort,
                remoteAddress: remoteAddress,
                remotePort: remotePort,
                routeStatus: status
            }
        );
    }

    return routes;
}

/// Returns a vetor containing all network interfaces found in sysfs
pub fn networkInterfaces() -> Vec<NetworkInterface> {
    let baseDirectory = "/sys/class/net";
    let mut interfaces = Vec::new();
    
    let virtualInterfaces = {
        let mut ifaces = Vec::<String>::new();

        let virtualInterfaces = fs::read_dir("/sys/devices/virtual/net");
        if !virtualInterfaces.is_err() {
            for iface in virtualInterfaces.unwrap() {
                if iface.is_err() {
                    continue;
                }

                let mut interfaceEntry = iface.unwrap();

                if interfaceEntry.path().is_dir() {
                    ifaces.push(interfaceEntry.file_name().to_str().unwrap().to_string());
                }
            }
        }

        ifaces
    };

    let interfacesEntries = fs::read_dir(baseDirectory);
    if interfacesEntries.is_err() {
        return Vec::new();
    }

    for directoryEntry in interfacesEntries.unwrap() {
        if directoryEntry.is_err() {
            continue;
        }

        let entry = directoryEntry.unwrap();
        if !entry.path().is_dir() {
            continue;
        }

        let name = entry.file_name().to_str().unwrap().to_string();
        let path = entry.path().to_str().unwrap().to_string();

        let mac = fs::read_to_string(format!("{}/address", path)).unwrap().trim().to_string();

        let mut interfaceType = InterfaceType::Physical;
        
        if virtualInterfaces.contains(&name) {
            interfaceType = InterfaceType::Virtual;
        }
        
        /*let directoryContent = fs::read_dir(path).unwrap();

        for entry in directoryContent {
            if ["phydev", "phy80211"].contains(&entry.unwrap().file_name().to_str().unwrap()) {
                interfaceType = InterfaceType::Physical;
                break;
            }
        }*/

        interfaces.push(NetworkInterface {
            name: name,
            interfaceType: interfaceType,
            macAddress: mac
        });
    }

    return interfaces;
}

fn getRate() -> (usize, usize) {
    let stats = readFile("/proc/net/dev");

    let mut downloadRate = 0_usize;
    let mut uploadRate = 0_usize;

    for line in stats.split('\n') {
        if line.contains(':') {
            let splitted = {
                let tmp = line.split(" ");

                let mut data = Vec::<usize>::new();
                for chunk in tmp {

                    if !chunk.is_empty() && !chunk.contains(' ') && !chunk.contains(":") {
                        data.push(chunk.parse().unwrap_or(0));
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

/// Returns current network rate (downlaod and upload), expressed in bytes
pub fn networkRate() -> NetworkRate {
    linuxCheck();

    let (downBefore, upBefore) = getRate();
    thread::sleep(Duration::from_millis(500));
    let (downAfter, upAfter) = getRate();

    let downloadRate: f32 = ((downAfter - downBefore) as f32) / 0.5_f32;
    let uploadRate: f32 = ((upAfter - upBefore) as f32) / 0.5_f32;

    return NetworkRate {
        download: downloadRate,
        upload: uploadRate,
    };
}

/// Returns a list of each internal network route
pub fn networkRoutes() -> Vec<NetworkRoute> {
    linuxCheck();
    let mut routes: Vec<NetworkRoute> = Vec::<NetworkRoute>::new();

    routes.append(
        &mut getRoutes(readFile("/proc/net/tcp"), ".", RouteType::TCP)
    );

    routes.append(
        &mut getRoutes(readFile("/proc/net/udp"), ".", RouteType::UDP)
    );

    routes.append(
        &mut getRoutes(readFile("/proc/net/tcp6"), ":", RouteType::TCP6)
    );

    routes.append(
        &mut getRoutes(readFile("/proc/net/udp6"), ":", RouteType::UDP6)
    );

    return routes;
}