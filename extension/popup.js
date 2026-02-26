document.addEventListener("DOMContentLoaded", async () => {
  const toggle = document.getElementById("intercept-toggle");
  const connDot = document.getElementById("conn-dot");
  const connText = document.getElementById("conn-text");
  const connHelp = document.getElementById("conn-help");

  // Load saved state
  const { enabled } = await chrome.storage.local.get(["enabled"]);
  toggle.checked = enabled !== false; // default true

  // Listen for changes
  toggle.addEventListener("change", (e) => {
    chrome.storage.local.set({ enabled: e.target.checked });
  });

  // Check connection to local server
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 2000);
    
    // Ping the local API server
    const response = await fetch("http://127.0.0.1:4142/api/ping", {
      signal: controller.signal
    });
    
    clearTimeout(timeoutId);
    
    if (response.ok) {
      connDot.className = "dot connected";
      connText.innerText = "Connected";
      connText.style.color = "#10b981";
      connHelp.style.display = "none";
    } else {
      throw new Error("HTTP " + response.status);
    }
  } catch (error) {
    connDot.className = "dot disconnected";
    connText.innerText = "Disconnected";
    connText.style.color = "#ef4444";
    connHelp.style.display = "block";
    console.error("Connection check failed:", error);
  }
});
