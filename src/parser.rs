use std::fs;
use std::path::Path;
use crate::models::*;

fn skip_whitespace_and_comments(input: &str) -> &str {
    let mut pos = 0;
    let bytes = input.as_bytes();
    while pos < bytes.len() {
        match bytes[pos] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                pos += 1;
            }
            b'#' => {
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
            }
            b'/' if pos + 1 < bytes.len() && bytes[pos + 1] == b'/' => {
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
            }
            _ => break,
        }
    }
    &input[pos..]
}

fn find_next_token(input: &str) -> Option<(&str, &str)> {
    let rest = skip_whitespace_and_comments(input);
    if rest.is_empty() {
        return None;
    }
    let end_pos = rest.find(|c: char| !c.is_alphanumeric() && c != '-' && c != '.' && c != '_' && c != ':')?;
    if end_pos == 0 {
        None
    } else {
        Some((&rest[end_pos..], &rest[..end_pos]))
    }
}

fn parse_option(input: &str) -> Option<(&str, DhcpOption)> {
    let rest = skip_whitespace_and_comments(input);
    if !rest.starts_with("option ") {
        return None;
    }
    let rest = &rest[7..];
    let rest = skip_whitespace_and_comments(rest);
    
    let (after_name, name) = find_next_token(rest)?;
    let rest = skip_whitespace_and_comments(after_name);
    
    if !rest.starts_with('=') {
        return None;
    }
    let rest = skip_whitespace_and_comments(&rest[1..]);
    
    let semicolon_pos = rest.find(';')?;
    let value = rest[..semicolon_pos].trim().to_string();
    let remaining = skip_whitespace_and_comments(&rest[semicolon_pos + 1..]);
    
    Some((remaining, DhcpOption { name: name.to_string(), value }))
}

fn parse_range(input: &str) -> Option<(&str, Range)> {
    let rest = skip_whitespace_and_comments(input);
    if !rest.starts_with("range ") {
        return None;
    }
    let rest = &rest[6..];
    let rest = skip_whitespace_and_comments(rest);
    
    let (after_start, start) = find_next_token(rest)?;
    let rest = skip_whitespace_and_comments(after_start);
    let (after_end, end_part) = find_next_token(rest)?;
    
    let end = end_part.trim_end_matches(';').to_string();
    let after_end = after_end.trim_start_matches(';');
    let remaining = skip_whitespace_and_comments(after_end);
    
    Some((remaining, Range { start: start.to_string(), end }))
}

fn parse_host(input: &str) -> Option<(&str, Host)> {
    let rest = skip_whitespace_and_comments(input);
    if !rest.starts_with("host ") {
        return None;
    }
    let rest = &rest[5..];
    let rest = skip_whitespace_and_comments(rest);
    
    let (_, name) = find_next_token(rest)?;
    let rest = skip_whitespace_and_comments(&rest[name.len()..]);
    
    if !rest.starts_with('{') {
        return None;
    }
    let rest = &rest[1..];

    let mut host = Host {
        name: name.to_string(),
        hardware_address: None,
        fixed_address: None,
        options: Vec::new(),
    };

    let mut remaining = rest;
    loop {
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if remaining.starts_with("hardware") {
            let after_hw = skip_whitespace_and_comments(&remaining[8..]);
            let (after_eth, _) = find_next_token(after_hw)?;
            if after_eth.starts_with("ethernet") {
                let after_eth = skip_whitespace_and_comments(&after_eth[8..]);
                let (after_mac, mac) = find_next_token(after_eth)?;
                host.hardware_address = Some(mac.trim_end_matches(';').to_string());
                remaining = skip_whitespace_and_comments(after_mac.trim_start_matches(';'));
                continue;
            }
        }

        if remaining.starts_with("fixed-address") {
            let after_fa = skip_whitespace_and_comments(&remaining[13..]);
            let (after_addr, addr) = find_next_token(after_fa)?;
            host.fixed_address = Some(addr.trim_end_matches(';').to_string());
            remaining = skip_whitespace_and_comments(after_addr.trim_start_matches(';'));
            continue;
        }

        if let Some((next, opt)) = parse_option(remaining) {
            host.options.push(opt);
            remaining = next;
            continue;
        }

        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some(semi) = remaining.find(';') {
            remaining = &remaining[semi + 1..];
            continue;
        }

        if let Some(pos) = remaining.find(|c| c == '}' || c == '{') {
            remaining = &remaining[pos + 1..];
            continue;
        }

        if remaining.is_empty() || remaining.len() < 2 {
            break;
        }
        remaining = &remaining[1..];
    }

    Some((remaining, host))
}

fn parse_subnet(input: &str) -> Option<(&str, Subnet)> {
    let rest = skip_whitespace_and_comments(input);
    if !rest.starts_with("subnet ") {
        return None;
    }
    let rest = &rest[7..];
    let rest = skip_whitespace_and_comments(rest);

    let (after_network, network) = find_next_token(rest)?;
    let rest = skip_whitespace_and_comments(after_network);

    if !rest.starts_with("netmask") {
        return None;
    }
    let rest = skip_whitespace_and_comments(&rest[7..]);
    let (after_netmask, netmask) = find_next_token(rest)?;
    let rest = skip_whitespace_and_comments(after_netmask);

    if !rest.starts_with('{') {
        return None;
    }
    let rest = &rest[1..];

    let mut subnet = Subnet {
        network: network.to_string(),
        netmask: netmask.to_string(),
        options: Vec::new(),
        pools: Vec::new(),
        ranges: Vec::new(),
        host_declarations: Vec::new(),
    };

    let mut remaining = rest;
    loop {
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some((next, opt)) = parse_option(remaining) {
            subnet.options.push(opt);
            remaining = next;
            continue;
        }

        if let Some((next, range)) = parse_range(remaining) {
            subnet.ranges.push(range);
            remaining = next;
            continue;
        }

        if let Some((next, host)) = parse_host(remaining) {
            subnet.host_declarations.push(host);
            remaining = next;
            continue;
        }

        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some(semi) = remaining.find(';') {
            remaining = &remaining[semi + 1..];
            continue;
        }

        if let Some(pos) = remaining.find(|c| c == '}') {
            remaining = &remaining[pos + 1..];
            continue;
        }

        if remaining.is_empty() {
            break;
        }
        remaining = &remaining[1..];
    }

    Some((remaining, subnet))
}

fn parse_shared_network(input: &str) -> Option<(&str, SharedNetwork)> {
    let rest = skip_whitespace_and_comments(input);
    if !rest.starts_with("shared-network ") {
        return None;
    }
    let rest = &rest[15..];
    let rest = skip_whitespace_and_comments(rest);

    let (after_name, name) = find_next_token(rest)?;
    let rest = skip_whitespace_and_comments(after_name);
    if !rest.starts_with('{') {
        return None;
    }
    let rest = &rest[1..];

    let mut shared = SharedNetwork {
        name: name.to_string(),
        options: Vec::new(),
        subnets: Vec::new(),
        host_declarations: Vec::new(),
    };

    let mut remaining = rest;
    loop {
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some((next, opt)) = parse_option(remaining) {
            shared.options.push(opt);
            remaining = next;
            continue;
        }

        if let Some((next, subnet)) = parse_subnet(remaining) {
            shared.subnets.push(subnet);
            remaining = next;
            continue;
        }

        if let Some((next, host)) = parse_host(remaining) {
            shared.host_declarations.push(host);
            remaining = next;
            continue;
        }

        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some(semi) = remaining.find(';') {
            remaining = &remaining[semi + 1..];
            continue;
        }

        if let Some(pos) = remaining.find(|c| c == '}') {
            remaining = &remaining[pos + 1..];
            continue;
        }

        if remaining.is_empty() {
            break;
        }
        remaining = &remaining[1..];
    }

    Some((remaining, shared))
}

fn parse_group(input: &str) -> Option<(&str, Group)> {
    let rest = skip_whitespace_and_comments(input);
    if !rest.starts_with("group") {
        return None;
    }
    let rest = &rest[5..];
    let rest = skip_whitespace_and_comments(rest);

    let name: Option<String>;
    let rest2: &str;

    if rest.starts_with('{') {
        name = None;
        rest2 = rest;
    } else {
        let (after_name, group_name) = find_next_token(rest)?;
        name = Some(group_name.to_string());
        rest2 = skip_whitespace_and_comments(after_name);
    }

    if !rest2.starts_with('{') {
        return None;
    }
    let rest = &rest2[1..];

    let mut group = Group {
        name,
        options: Vec::new(),
        hosts: Vec::new(),
    };

    let mut remaining = rest;
    loop {
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some((next, opt)) = parse_option(remaining) {
            group.options.push(opt);
            remaining = next;
            continue;
        }

        if let Some((next, host)) = parse_host(remaining) {
            group.hosts.push(host);
            remaining = next;
            continue;
        }

        if remaining.starts_with('}') {
            remaining = skip_whitespace_and_comments(&remaining[1..]);
            break;
        }

        if let Some(semi) = remaining.find(';') {
            remaining = &remaining[semi + 1..];
            continue;
        }

        if let Some(pos) = remaining.find(|c| c == '}') {
            remaining = &remaining[pos + 1..];
            continue;
        }

        if remaining.is_empty() {
            break;
        }
        remaining = &remaining[1..];
    }

    Some((remaining, group))
}

pub fn parse_dhcpd_conf(content: &str) -> DhcpConfig {
    let mut config = DhcpConfig::default();
    let mut remaining = content;

    loop {
        remaining = skip_whitespace_and_comments(remaining);
        if remaining.is_empty() {
            break;
        }

        if remaining.starts_with("option ") {
            if let Some((next, opt)) = parse_option(remaining) {
                config.global_options.push(opt);
                remaining = next;
                continue;
            }
        }

        if remaining.starts_with("subnet ") {
            if let Some((next, subnet)) = parse_subnet(remaining) {
                config.subnets.push(subnet);
                remaining = next;
                continue;
            }
        }

        if remaining.starts_with("host ") {
            if let Some((next, host)) = parse_host(remaining) {
                config.hosts.push(host);
                remaining = next;
                continue;
            }
        }

        if remaining.starts_with("shared-network ") {
            if let Some((next, shared)) = parse_shared_network(remaining) {
                config.shared_networks.push(shared);
                remaining = next;
                continue;
            }
        }

        if remaining.starts_with("group") {
            if let Some((next, group)) = parse_group(remaining) {
                config.groups.push(group);
                remaining = next;
                continue;
            }
        }

        if let Some(semi) = remaining.find(';') {
            remaining = &remaining[semi + 1..];
            continue;
        }

        if let Some(pos) = remaining.find(|c| c == '{' || c == '}') {
            remaining = &remaining[pos + 1..];
            continue;
        }

        if remaining.is_empty() {
            break;
        }
        remaining = &remaining[1..];
    }

    config
}

pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<DhcpConfig, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(parse_dhcpd_conf(&content))
}

pub fn serialize_to_string(config: &DhcpConfig) -> String {
    let mut output = String::new();

    for opt in &config.global_options {
        output.push_str(&format!("option {} {};\n", opt.name, opt.value));
    }
    if !config.global_options.is_empty() {
        output.push('\n');
    }

    for subnet in &config.subnets {
        output.push_str(&format!("subnet {} netmask {} {{\n", subnet.network, subnet.netmask));
        for opt in &subnet.options {
            output.push_str(&format!("  option {} {};\n", opt.name, opt.value));
        }
        for range in &subnet.ranges {
            output.push_str(&format!("  range {} {};\n", range.start, range.end));
        }
        for host in &subnet.host_declarations {
            output.push_str(&format!("  host {} {{\n", host.name));
            if let Some(hw) = &host.hardware_address {
                output.push_str(&format!("    hardware ethernet {};\n", hw));
            }
            if let Some(fa) = &host.fixed_address {
                output.push_str(&format!("    fixed-address {};\n", fa));
            }
            for opt in &host.options {
                output.push_str(&format!("    option {} {};\n", opt.name, opt.value));
            }
            output.push_str("  }\n");
        }
        output.push_str("}\n\n");
    }

    for host in &config.hosts {
        output.push_str(&format!("host {} {{\n", host.name));
        if let Some(hw) = &host.hardware_address {
            output.push_str(&format!("  hardware ethernet {};\n", hw));
        }
        if let Some(fa) = &host.fixed_address {
            output.push_str(&format!("  fixed-address {};\n", fa));
        }
        for opt in &host.options {
            output.push_str(&format!("  option {} {};\n", opt.name, opt.value));
        }
        output.push_str("}\n\n");
    }

    for shared in &config.shared_networks {
        output.push_str(&format!("shared-network {} {{\n", shared.name));
        for opt in &shared.options {
            output.push_str(&format!("  option {} {};\n", opt.name, opt.value));
        }
        for subnet in &shared.subnets {
            output.push_str(&format!("  subnet {} netmask {} {{\n", subnet.network, subnet.netmask));
            for opt in &subnet.options {
                output.push_str(&format!("    option {} {};\n", opt.name, opt.value));
            }
            for range in &subnet.ranges {
                output.push_str(&format!("    range {} {};\n", range.start, range.end));
            }
            output.push_str("  }\n");
        }
        output.push_str("}\n\n");
    }

    for group in &config.groups {
        if let Some(name) = &group.name {
            output.push_str(&format!("group {} {{\n", name));
        } else {
            output.push_str("group {\n");
        }
        for opt in &group.options {
            output.push_str(&format!("  option {} {};\n", opt.name, opt.value));
        }
        for host in &group.hosts {
            output.push_str(&format!("  host {} {{\n", host.name));
            if let Some(hw) = &host.hardware_address {
                output.push_str(&format!("    hardware ethernet {};\n", hw));
            }
            if let Some(fa) = &host.fixed_address {
                output.push_str(&format!("    fixed-address {};\n", fa));
            }
            for opt in &host.options {
                output.push_str(&format!("    option {} {};\n", opt.name, opt.value));
            }
            output.push_str("  }\n");
        }
        output.push_str("}\n\n");
    }

    output
}
