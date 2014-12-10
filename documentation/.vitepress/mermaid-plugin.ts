export default function mermaidPlugin(md, options) {
    // Setup Mermaid - this will be initialized on the client side
    // Mermaid.initialize(Object.assign({ securityLevel: "loose" }, options)); // Remove this line

    function getLangName(info) {
        return info.split(/\s+/g)[0];
    }

    let defaultFenceRenderer = md.renderer.rules.fence;

    function customFenceRenderer(tokens, idx, options, env, slf) {
        let token = tokens[idx];
        let info = token.info.trim();
        let langName = info ? getLangName(info) : "";

        console.log('mermaid-plugin: customFenceRenderer called for token:', token);
        console.log('mermaid-plugin: langName:', langName);

        if (["mermaid", "{mermaid}"].indexOf(langName) === -1) {
            if (defaultFenceRenderer !== undefined) {
                return defaultFenceRenderer(tokens, idx, options, env, slf);
            }
            return "";
        }

        const containerId = `mermaid-${idx}`;
        const mermaidCode = token.content;

        console.log('mermaid-plugin: Mermaid code detected:', mermaidCode);

        // Return a div with the mermaid code as a data attribute
        const renderedHtml = `<div class="mermaid" id="${containerId}" data-mermaid-code="${encodeURIComponent(mermaidCode)}"></div>`;
        console.log('mermaid-plugin: Rendered Mermaid HTML:', renderedHtml);
        return renderedHtml;
    }

    md.renderer.rules.fence = customFenceRenderer;
}