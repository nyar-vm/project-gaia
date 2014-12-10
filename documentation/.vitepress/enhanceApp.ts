import { EnhanceAppContext } from 'vitepress';
import mermaid from 'mermaid';

export default function enhanceApp({ app, router, siteData }: EnhanceAppContext) {
  console.log('enhanceApp: Function called');
  
  if (typeof window !== 'undefined') {
    console.log('enhanceApp: Running in browser environment');
    console.log('enhanceApp: Mermaid loaded successfully');
    
    mermaid.initialize({
      startOnLoad: false,
      theme: 'default',
      securityLevel: 'loose',
    });

    // 页面加载完成后渲染 Mermaid 图表
    const renderMermaid = () => {
      console.log('renderMermaid: Function called');
      const mermaidElements = document.querySelectorAll('div.mermaid[data-mermaid-code]');
      console.log(`renderMermaid: Found ${mermaidElements.length} Mermaid elements`);
      
      if (mermaidElements.length === 0) {
        console.log('renderMermaid: No Mermaid elements found, checking all div.mermaid elements');
        const allMermaidDivs = document.querySelectorAll('div.mermaid');
        console.log(`renderMermaid: Found ${allMermaidDivs.length} div.mermaid elements total`);
        allMermaidDivs.forEach((div, i) => {
          console.log(`renderMermaid: div.mermaid[${i}] has attributes:`, Array.from(div.attributes).map(attr => `${attr.name}="${attr.value}"`));
        });
      }
      
      mermaidElements.forEach((element, index) => {
        console.log(`renderMermaid: Processing element ${index}`);
        const code = decodeURIComponent(element.getAttribute('data-mermaid-code') || '');
        console.log(`renderMermaid: Decoded code for element ${index}:`, code);
        
        if (code) {
          mermaid.render(`mermaid-${Date.now()}-${index}`, code).then((result) => {
            console.log(`renderMermaid: Successfully rendered element ${index}`);
            element.innerHTML = result.svg;
          }).catch((error) => {
            console.error(`renderMermaid: Error rendering element ${index}:`, error);
            element.innerHTML = `<pre>Mermaid Error: ${error.message}</pre>`;
          });
        } else {
          console.log(`renderMermaid: No code found for element ${index}`);
        }
      });
    };

    // 等待 DOM 完全加载
    if (document.readyState === 'loading') {
      console.log('enhanceApp: DOM still loading, waiting for DOMContentLoaded');
      document.addEventListener('DOMContentLoaded', renderMermaid);
    } else {
      console.log('enhanceApp: DOM already loaded, rendering immediately');
      renderMermaid();
    }
    
    // 也在 window.onload 时再次尝试
    window.addEventListener('load', () => {
      console.log('enhanceApp: Window loaded, rendering again');
      renderMermaid();
    });
  } else {
    console.log('enhanceApp: Running in server environment, skipping Mermaid initialization');
  }
}