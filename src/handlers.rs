use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::models::DhcpConfig;
use crate::parser::{parse_from_file, serialize_to_string};

#[derive(Clone)]
pub struct AppState {
    pub config_path: PathBuf,
}

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_editor))
        .route("/api/config", get(get_config))
        .route("/api/config", post(save_config))
        .route("/health", get(health_check))
        .with_state(Arc::new(state))
}

async fn serve_editor() -> Html<String> {
    Html(EDITOR_HTML.to_string())
}

async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DhcpConfig>, (StatusCode, String)> {
    parse_from_file(&state.config_path)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn save_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<DhcpConfig>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let content = serialize_to_string(&config);
    fs::write(&state.config_path, &content).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write config: {}", e))
    })?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Configuration saved successfully"
    })))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

static EDITOR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DHCP Config Editor</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; color: #333; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        header { background: #2c3e50; color: white; padding: 20px; margin-bottom: 20px; border-radius: 8px; display: flex; justify-content: space-between; align-items: center; }
        h1 { font-size: 1.5rem; }
        .btn { padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; font-size: 0.9rem; transition: background 0.2s; }
        .btn-primary { background: #3498db; color: white; }
        .btn-primary:hover { background: #2980b9; }
        .btn-success { background: #27ae60; color: white; }
        .btn-success:hover { background: #219a52; }
        .btn-danger { background: #e74c3c; color: white; }
        .btn-danger:hover { background: #c0392b; }
        .btn-sm { padding: 5px 10px; font-size: 0.8rem; }
        .section { background: white; border-radius: 8px; padding: 20px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px; padding-bottom: 10px; border-bottom: 1px solid #eee; }
        .section h2 { font-size: 1.2rem; color: #2c3e50; }
        .card { border: 1px solid #e0e0e0; border-radius: 6px; padding: 15px; margin-bottom: 10px; background: #fafafa; }
        .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
        .card h3 { font-size: 1rem; color: #34495e; }
        .card p { font-size: 0.85rem; color: #666; margin: 5px 0; }
        .card-actions { display: flex; gap: 5px; }
        .options-list { margin-top: 10px; padding-left: 15px; }
        .option-item { font-family: monospace; font-size: 0.85rem; padding: 3px 0; color: #555; }
        .add-form { background: #f8f9fa; padding: 15px; border-radius: 6px; margin-top: 10px; display: none; }
        .add-form.active { display: block; }
        .form-group { margin-bottom: 10px; }
        .form-group label { display: block; font-size: 0.85rem; margin-bottom: 3px; color: #666; }
        .form-group input { width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 0.9rem; }
        .form-row { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 10px; }
        .toast { position: fixed; bottom: 20px; right: 20px; padding: 15px 25px; border-radius: 6px; color: white; font-weight: 500; opacity: 0; transition: opacity 0.3s; }
        .toast.success { background: #27ae60; opacity: 1; }
        .toast.error { background: #e74c3c; opacity: 1; }
        .empty-state { text-align: center; padding: 40px; color: #999; }
        .empty-state p { margin-bottom: 15px; }
        .collapsible { cursor: pointer; }
        .collapsible::before { content: '▼'; margin-right: 8px; font-size: 0.7rem; }
        .collapsible.collapsed::before { content: '▶'; }
        .collapsible-content { margin-top: 10px; }
        .collapsible.collapsed + .collapsible-content { display: none; }
        .modal { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
        .modal-content { background: white; padding: 30px; border-radius: 8px; max-width: 600px; width: 90%; max-height: 80vh; overflow-y: auto; }
        .modal-header { display: flex; justify-content: space-between; margin-bottom: 20px; }
        .modal h2 { font-size: 1.3rem; }
        .modal-close { background: none; border: none; font-size: 1.5rem; cursor: pointer; }
        .modal-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>DHCP Config Editor</h1>
            <div>
                <button class="btn btn-primary" onclick="loadConfig()">Refresh</button>
                <button class="btn btn-success" onclick="saveConfig()">Save Changes</button>
            </div>
        </header>

        <div id="global-options" class="section">
            <div class="section-header">
                <h2>Global Options</h2>
                <button class="btn btn-primary btn-sm" onclick="showAddOption('global')">+ Add Option</button>
            </div>
            <div id="global-options-list"></div>
            <div id="global-option-form" class="add-form">
                <div class="form-row">
                    <div class="form-group">
                        <label>Option Name</label>
                        <input type="text" id="opt-name-global" placeholder="e.g., domain-name-servers">
                    </div>
                    <div class="form-group">
                        <label>Value</label>
                        <input type="text" id="opt-value-global" placeholder="e.g., 8.8.8.8, 8.8.4.4">
                    </div>
                </div>
                <button class="btn btn-success btn-sm" onclick="addOption('global')">Add</button>
                <button class="btn btn-danger btn-sm" onclick="hideAddOption('global')">Cancel</button>
            </div>
        </div>

        <div id="subnets-section" class="section">
            <div class="section-header">
                <h2>Subnets</h2>
                <button class="btn btn-primary btn-sm" onclick="showAddSubnet()">+ Add Subnet</button>
            </div>
            <div id="subnets-list"></div>
        </div>

        <div id="hosts-section" class="section">
            <div class="section-header">
                <h2>Host Declarations</h2>
                <button class="btn btn-primary btn-sm" onclick="showAddHost()">+ Add Host</button>
            </div>
            <div id="hosts-list"></div>
        </div>

        <div id="shared-networks-section" class="section">
            <div class="section-header">
                <h2>Shared Networks</h2>
                <button class="btn btn-primary btn-sm" onclick="showAddSharedNetwork()">+ Add Shared Network</button>
            </div>
            <div id="shared-networks-list"></div>
        </div>
    </div>

    <div id="toast" class="toast"></div>

    <script>
        let config = { global_options: [], subnets: [], hosts: [], shared_networks: [], groups: [] };

        async function loadConfig() {
            try {
                const res = await fetch('/api/config');
                config = await res.json();
                render();
            } catch (e) {
                showToast('Failed to load config: ' + e.message, 'error');
            }
        }

        async function saveConfig() {
            try {
                const res = await fetch('/api/config', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(config)
                });
                if (res.ok) {
                    showToast('Configuration saved successfully!', 'success');
                } else {
                    showToast('Failed to save configuration', 'error');
                }
            } catch (e) {
                showToast('Error: ' + e.message, 'error');
            }
        }

        function render() {
            renderGlobalOptions();
            renderSubnets();
            renderHosts();
            renderSharedNetworks();
        }

        function renderGlobalOptions() {
            const list = document.getElementById('global-options-list');
            if (!config.global_options.length) {
                list.innerHTML = '<div class="empty-state"><p>No global options defined</p></div>';
                return;
            }
            list.innerHTML = config.global_options.map((opt, i) => `
                <div class="card">
                    <div class="card-header">
                        <h3>${opt.name}</h3>
                        <button class="btn btn-danger btn-sm" onclick="removeGlobalOption(${i})">Remove</button>
                    </div>
                    <p><strong>Value:</strong> ${opt.value}</p>
                </div>
            `).join('');
        }

        function renderSubnets() {
            const list = document.getElementById('subnets-list');
            if (!config.subnets.length) {
                list.innerHTML = '<div class="empty-state"><p>No subnets defined</p></div>';
                return;
            }
            list.innerHTML = config.subnets.map((subnet, i) => `
                <div class="card">
                    <div class="card-header">
                        <h3>Subnet: ${subnet.network}/${subnet.netmask}</h3>
                        <div>
                            <button class="btn btn-primary btn-sm" onclick="showAddHostInSubnet(${i})">+ Host</button>
                            <button class="btn btn-primary btn-sm" onclick="showAddOptionInSubnet(${i})">+ Option</button>
                            <button class="btn btn-danger btn-sm" onclick="removeSubnet(${i})">Remove</button>
                        </div>
                    </div>
                    ${subnet.ranges.length ? `<p><strong>Ranges:</strong> ${subnet.ranges.map(r => `${r.start} - ${r.end}`).join(', ')}</p>` : ''}
                    <div class="options-list">
                        ${subnet.options.map(o => `<div class="option-item">option ${o.name} ${o.value};</div>`).join('')}
                    </div>
                    ${subnet.host_declarations.length ? `
                        <div style="margin-top: 10px; padding-left: 15px;">
                            <strong>Hosts:</strong>
                            ${subnet.host_declarations.map(h => `<div class="option-item">host ${h.name} { ${h.fixed_address || 'no fixed-address'} }</div>`).join('')}
                        </div>
                    ` : ''}
                </div>
            `).join('');
        }

        function renderHosts() {
            const list = document.getElementById('hosts-list');
            if (!config.hosts.length) {
                list.innerHTML = '<div class="empty-state"><p>No standalone host declarations</p></div>';
                return;
            }
            list.innerHTML = config.hosts.map((host, i) => `
                <div class="card">
                    <div class="card-header">
                        <h3>Host: ${host.name}</h3>
                        <button class="btn btn-danger btn-sm" onclick="removeHost(${i})">Remove</button>
                    </div>
                    ${host.hardware_address ? `<p><strong>Hardware:</strong> ${host.hardware_address}</p>` : ''}
                    ${host.fixed_address ? `<p><strong>Fixed Address:</strong> ${host.fixed_address}</p>` : ''}
                    <div class="options-list">
                        ${host.options.map(o => `<div class="option-item">option ${o.name} ${o.value};</div>`).join('')}
                    </div>
                </div>
            `).join('');
        }

        function renderSharedNetworks() {
            const list = document.getElementById('shared-networks-list');
            if (!config.shared_networks.length) {
                list.innerHTML = '<div class="empty-state"><p>No shared networks defined</p></div>';
                return;
            }
            list.innerHTML = config.shared_networks.map((net, i) => `
                <div class="card">
                    <div class="card-header">
                        <h3>Shared Network: ${net.name}</h3>
                        <button class="btn btn-danger btn-sm" onclick="removeSharedNetwork(${i})">Remove</button>
                    </div>
                    <p>${net.subnets.length} subnet(s), ${net.host_declarations.length} host(s)</p>
                </div>
            `).join('');
        }

        function showAddOption(target) {
            document.getElementById(target + '-option-form').classList.add('active');
        }

        function hideAddOption(target) {
            document.getElementById(target + '-option-form').classList.remove('active');
            document.getElementById('opt-name-' + target).value = '';
            document.getElementById('opt-value-' + target).value = '';
        }

        function addOption(target) {
            const name = document.getElementById('opt-name-' + target).value.trim();
            const value = document.getElementById('opt-value-' + target).value.trim();
            if (!name || !value) {
                showToast('Please fill in both name and value', 'error');
                return;
            }
            config.global_options.push({ name, value });
            hideAddOption(target);
            renderGlobalOptions();
        }

        function removeGlobalOption(index) {
            config.global_options.splice(index, 1);
            renderGlobalOptions();
        }

        function removeSubnet(index) {
            config.subnets.splice(index, 1);
            renderSubnets();
        }

        function removeHost(index) {
            config.hosts.splice(index, 1);
            renderHosts();
        }

        function removeSharedNetwork(index) {
            config.shared_networks.splice(index, 1);
            renderSharedNetworks();
        }

        function showAddSubnet() {
            const network = prompt('Enter network address (e.g., 192.168.1.0):');
            if (!network) return;
            const netmask = prompt('Enter netmask (e.g., 255.255.255.0):');
            if (!netmask) return;
            config.subnets.push({
                network, netmask,
                options: [],
                pools: [],
                ranges: [],
                host_declarations: []
            });
            renderSubnets();
        }

        function showAddHost() {
            const name = prompt('Enter host name:');
            if (!name) return;
            const hardware = prompt('Enter hardware address (optional):');
            const fixedAddress = prompt('Enter fixed address (optional):');
            const host = {
                name,
                hardware_address: hardware || null,
                fixed_address: fixedAddress || null,
                options: []
            };
            config.hosts.push(host);
            renderHosts();
        }

        function showAddSharedNetwork() {
            const name = prompt('Enter shared network name:');
            if (!name) return;
            config.shared_networks.push({
                name,
                options: [],
                subnets: [],
                host_declarations: []
            });
            renderSharedNetworks();
        }

        function showToast(message, type) {
            const toast = document.getElementById('toast');
            toast.textContent = message;
            toast.className = 'toast ' + type;
            setTimeout(() => { toast.className = 'toast'; }, 3000);
        }

        loadConfig();
    </script>
</body>
</html>"#;
