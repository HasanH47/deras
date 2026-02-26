const SERVER_URL = "http://127.0.0.1:4142/api/add-download";

// Initialize state
chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.set({ enabled: true });

  // Create context menu for links
  chrome.contextMenus.create({
    id: "download_with_deras",
    title: "Download with Deras",
    contexts: ["link"]
  });
});

// Handle Context Menu clicks
chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === "download_with_deras" && info.linkUrl) {
    sendToDeras(info.linkUrl);
  }
});

// Intercept standard downloads
chrome.downloads.onCreated.addListener(async (downloadItem) => {
  // Check if extension is enabled
  const { enabled } = await chrome.storage.local.get(["enabled"]);
  
  if (enabled && downloadItem.url && downloadItem.url.startsWith("http")) {
    // Prevent the browser from downloading the file natively
    chrome.downloads.cancel(downloadItem.id);
    console.log("Intercepted download:", downloadItem.url);
    
    // Forward the URL to the local Deras client
    sendToDeras(downloadItem.url);
  }
});

// Helper function to send HTTP POST to the local Axum server
async function sendToDeras(url) {
  try {
    const response = await fetch(SERVER_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ url })
    });

    if (!response.ok) {
      console.error("Deras API returned an error:", await response.text());
    } else {
      console.log("Successfully queued download in Deras.");
    }
  } catch (error) {
    console.error("Failed to connect to Deras local server. Is the app running?", error);
    // If Deras is not running, we could optionally restart the native browser download here,
    // or just show a notification to the user.
    chrome.notifications.create({
      type: "basic",
      iconUrl: "icon48.png",
      title: "Deras Companion",
      message: "Could not connect to Deras. Ensure the desktop app is running."
    });
  }
}
