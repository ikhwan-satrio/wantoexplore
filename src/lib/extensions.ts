function buildBridgeScript(
  pluginId: string,
  pluginCode: string,
  allowedActions: string[],
): string {
  return `
    <script>
      const pluginAPI = {
        allowedActions: ${JSON.stringify(allowedActions)},

        async callTauri(command, args = {}) {
          return new Promise((resolve, reject) => {
            const requestId = Math.random().toString(36).substr(2, 9);
            const handler = (event) => {
              if (event.data.requestId === requestId) {
                window.removeEventListener('message', handler);
                if (event.data.error) reject(new Error(event.data.error));
                else resolve(event.data.result);
              }
            };
            window.addEventListener('message', handler);
            window.parent.postMessage({ type: 'tauri-invoke', command, args, requestId }, '*');
          });
        },

        async readFile(path) { return this.callTauri('app_plugin_read_file', { pluginId: '${pluginId}', path }); },
        async writeFile(path, content) { return this.callTauri('app_plugin_write_file', { pluginId: '${pluginId}', path, content }); },
        async listDir(path) { return this.callTauri('app_plugin_list_dir', { pluginId: '${pluginId}', path }); },
        async getSelectedFiles() { return this.callTauri('get_selected_files'); },
        showToast(message) { window.parent.postMessage({ type: 'show-toast', message }, '*'); },
        async previewFile(path) { return this.callTauri('preview_file', { path }); },
      };

      window.pluginAPI = pluginAPI;

      try { ${pluginCode} }
      catch (e) {
        console.error('Plugin execution error:', e);
        window.parent.postMessage({ type: 'plugin-error', error: e.message }, '*');
      }
    <\/script>
  `;
}

export function buildSrcdoc(
  pluginCode: string,
  pluginId: string,
  allowedActions: string[],
): string {
  const bridge = buildBridgeScript(pluginId, pluginCode, allowedActions);
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'">
</head>
<body>
  ${bridge}
</body>
</html>`;
}
