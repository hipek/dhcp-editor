use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DhcpConfig {
    pub global_options: Vec<DhcpOption>,
    pub subnets: Vec<Subnet>,
    pub hosts: Vec<Host>,
    pub shared_networks: Vec<SharedNetwork>,
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subnet {
    pub network: String,
    pub netmask: String,
    pub options: Vec<DhcpOption>,
    pub pools: Vec<Pool>,
    pub ranges: Vec<Range>,
    pub host_declarations: Vec<Host>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedNetwork {
    pub name: String,
    pub options: Vec<DhcpOption>,
    pub subnets: Vec<Subnet>,
    pub host_declarations: Vec<Host>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub range: Option<Range>,
    pub options: Vec<DhcpOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub hardware_address: Option<String>,
    pub fixed_address: Option<String>,
    pub options: Vec<DhcpOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: Option<String>,
    pub options: Vec<DhcpOption>,
    pub hosts: Vec<Host>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpOption {
    pub name: String,
    pub value: String,
}

impl Default for Subnet {
    fn default() -> Self {
        Self {
            network: String::new(),
            netmask: String::new(),
            options: Vec::new(),
            pools: Vec::new(),
            ranges: Vec::new(),
            host_declarations: Vec::new(),
        }
    }
}

impl Default for SharedNetwork {
    fn default() -> Self {
        Self {
            name: String::new(),
            options: Vec::new(),
            subnets: Vec::new(),
            host_declarations: Vec::new(),
        }
    }
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            range: None,
            options: Vec::new(),
        }
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            start: String::new(),
            end: String::new(),
        }
    }
}

impl Default for Host {
    fn default() -> Self {
        Self {
            name: String::new(),
            hardware_address: None,
            fixed_address: None,
            options: Vec::new(),
        }
    }
}

impl Default for Group {
    fn default() -> Self {
        Self {
            name: None,
            options: Vec::new(),
            hosts: Vec::new(),
        }
    }
}

impl Default for DhcpOption {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
        }
    }
}
