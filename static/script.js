let roomListDiv = document.getElementById('room-list');
let messagesDiv = document.getElementById('messages');
let newMessageForm = document.getElementById('new-message');
let newRoomForm = document.getElementById('new-room');
let statusDiv = document.getElementById('status');

let roomTemplate = document.getElementById('room');
let messageTemplate = document.getElementById('message');

let messageField = newMessageForm.querySelector("#message");
let roomNameField = newRoomForm.querySelector("#name");
let logoutBtn = document.getElementById("logout-btn");

var STATE = {
  room: "lobby",
  connected: false,
}

// Set up the form handler.
newMessageForm.addEventListener("submit", (e) => {
  e.preventDefault();

  const room = STATE.room;
  const message = messageField.value;
  
  if (!message) return;

  if (chatSocket && chatSocket.readyState === WebSocket.OPEN) {
      chatSocket.send(JSON.stringify({ room, message }));
      messageField.value = "";
  }
});

// Set up the new room handler.
newRoomForm.addEventListener("submit", (e) => {
  e.preventDefault();

  const room = roomNameField.value;
  if (!room) return;

  roomNameField.value = "";
  if (!addRoom(room)) return;

  addMessage(room, "Rocket", `Look, your own "${room}" room! Nice.`, true);
})

logoutBtn.addEventListener("click", logout);

async function logout() {
  try {
    const response = await fetch("/logout", { method: "POST" });
    const data = await response.json();
    console.log(data.message);

    // Stop reconnect loop FIRST
    window.shouldReconnect = false;
    localStorage.removeItem("wasLoggedIn");  
    
    // Then close WebSocket
    if (window.chatSocket) {
      window.chatSocket.close();
      window.chatSocket = null;
    }

    // Clear user state
    STATE.username = null;
    STATE.color = null;

    resetUI();

    // Switch UI
    document.getElementById("chat").style.display = "none";
    document.getElementById("auth").style.display = "flex";

  } catch (err) {
    console.error("Logout failed:", err);
  }
}

async function sendAuthRequest(type) {
  const username = document.getElementById("auth-username").value;
  const password = document.getElementById("auth-password").value;

  if (!username || !password) {
    authStatus.textContent = "Username and password required";
    return;
  }

  const body = new URLSearchParams();
  body.append("username", username);
  body.append("password", password);

  const response = await fetch(`/${type}`, {
    method: "POST",
    body
  });

  const data = await response.json();
  authStatus.textContent = data.message;

  if (data.status.includes("succesfully logged in") || data.status === "ok") {
      STATE.username = username; // ⭐ store chosen color
      document.getElementById("chat").style.display = "flex";
      document.getElementById("auth").style.display = "none";
    
    window.shouldReconnect = true;
    localStorage.setItem("wasLoggedIn", "true"); 
    
    await uploadHistory();
    connectWebSocket();
  }
}

function setupAuth() {
  const loginBtn = document.getElementById("auth-login");
  const registerBtn = document.getElementById("auth-register");
  window.authStatus = document.getElementById("auth-status");

  loginBtn.addEventListener("click", (e) => {
    e.preventDefault();
    sendAuthRequest("login");
  });

  registerBtn.addEventListener("click", (e) => {
    e.preventDefault();
    sendAuthRequest("register");
  });
}

// Generate a color from a "hash" of a string. Thanks, internet.
function hashColor(str) {
  let hash = 0;
  for (var i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
    hash = hash & hash;
  }

  return `hsl(${hash % 360}, 100%, 70%)`;
}

// Add a new room `name` and change to it. Returns `true` if the room didn't
// already exist and false otherwise.
function addRoom(name) {
  if (STATE[name]) {
    changeRoom(name);
    return false;
  }

  var node = roomTemplate.content.cloneNode(true);
  var room = node.querySelector(".room");
  room.addEventListener("click", () => changeRoom(name));
  room.textContent = name;
  room.dataset.name = name;
  roomListDiv.appendChild(node);

  STATE[name] = [];
  changeRoom(name);
  return true;
}

// Change the current room to `name`, restoring its messages.
function changeRoom(name) {
  if (STATE.room == name) return;

  var newRoom = roomListDiv.querySelector(`.room[data-name='${name}']`);
  var oldRoom = roomListDiv.querySelector(`.room[data-name='${STATE.room}']`);
  if (!newRoom || !oldRoom) return;

  STATE.room = name;
  oldRoom.classList.remove("active");
  newRoom.classList.add("active");

  messagesDiv.querySelectorAll(".message").forEach((msg) => {
    messagesDiv.removeChild(msg)
  });

  STATE[name].forEach((data) => addMessage(name, data.username, data.message))
}

// Add `message` from `username` to `room`. If `push`, then actually store the
// message. If the current room is `room`, render the message.
function addMessage(room, username, message, push = false) {
  if (push) {
    STATE[room].push({ username, message })
  }

  if (STATE.room == room) {
    var node = messageTemplate.content.cloneNode(true);
    node.querySelector(".message .username").textContent = username;
    node.querySelector(".message .username").style.color = hashColor(username);
    node.querySelector(".message .text").textContent = message;
    messagesDiv.appendChild(node);
  }
}

// Set the connection status: `true` for connected, `false` for disconnected.
function setConnectedStatus(status) {
  STATE.connected = status;
  statusDiv.className = (status) ? "connected" : "reconnecting";
}
//Upload history
async function uploadHistory() {
  try {
    const response = await fetch("/history");
    if (!response.ok) {
      console.error("Failed to load history");
      return;
    }

    const history = await response.json();

    // history is an array of { room, username, message }
    history.forEach(msg => {
      // Ensure the room exists
      if (!STATE[msg.room]) {
        addRoom(msg.room);
      }

      // Add message to STATE and render if active
      addMessage(msg.room, msg.username, msg.message, true,);

    });

  } catch (err) {
    console.error("Error loading history:", err);
  }
}

function connectWebSocket() {
  console.log("Connecting to a WS");
  if (window.shouldReconnect === false) {
    console.log("Reconnect disabled (logged out)");
    return;
  }
  if (!STATE.username) {
    console.log("Not logged in, WS disabled");
    return;
  }

  let ws = new WebSocket(`ws://${window.location.hostname}:8080/ws`);

  ws.onopen = () => {
    setConnectedStatus(true);
    console.log("WebSocket connected");
    (async () => {
      const response = await fetch("/me")
      const data = await response.json();
      const [u, c] = data.message.split(":");
      STATE.username = u;
      STATE.color = c;
    }

    )
  };

  ws.onclose = () => {
    setConnectedStatus(false);
    console.log("WebSocket disconnected");
  };

  ws.onerror = () => {
    setConnectedStatus(false);
    console.log("WebSocket error");
    ws.close();
  };

  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
  
    if (!msg.room || !msg.username || !msg.message) return;
    
    if (!STATE[msg.room]) {
        addRoom(msg.room);
    }
    addMessage(msg.room, msg.username, msg.message, true);
  };

  window.chatSocket = ws;
}

function resetUI() {
  // Clear rooms
  roomListDiv.innerHTML = "";
  messagesDiv.innerHTML = "";

  // Reset STATE
  STATE = {
    room: "lobby",
    connected: false,
  };

  // Clear auth status
  authStatus.textContent = "";

  // Clear auth fields
  document.getElementById("auth-username").value = "";
  document.getElementById("auth-password").value = "";
}

async function checkSession() {
  try {
    const response = await fetch("/me");
    const data = await response.json();

    if (data.status === "ok") {
      const username = data.message;

      STATE.username = username;

      window.shouldReconnect = true;

      document.getElementById("chat").style.display = "flex";
      document.getElementById("auth").style.display = "none";

      await uploadHistory();
      connectWebSocket();
    } else {
      window.shouldReconnect = false;   // ⭐ prevent reconnect spam
    }
  } catch (err) {
    console.error("Session check failed:", err);
    window.shouldReconnect = false;     // ⭐ also disable on error
  }
}

// Let's go! Initialize the world.
async function init() {
  const wasLoggedIn = localStorage.getItem("wasLoggedIn") === "true";
 
  if (wasLoggedIn) {
    await checkSession();   // ⭐ only on refresh after login
  }
  
  if (!STATE.username) {
    document.getElementById("chat").style.display = "none";
    document.getElementById("auth").style.display = "flex";
    document.getElementById("auth").style.flexDirection = "column";
    return;
  }
}

setupAuth();
init();
