let roomListDiv = document.getElementById('room-list');
let messagesDiv = document.getElementById('messages');
let newMessageForm = document.getElementById('new-message');
let newRoomForm = document.getElementById('new-room');
let statusDiv = document.getElementById('status');

let roomTemplate = document.getElementById('room');
let messageTemplate = document.getElementById('message');

let messageField = newMessageForm.querySelector("#message");
let usernameField = newMessageForm.querySelector("#username");
let roomNameField = newRoomForm.querySelector("#name");


var STATE = {
  room: "lobby",
  rooms: {},
  connected: false,
}
// listeners 

// Set up the form handler.
newMessageForm.addEventListener("submit", (e) => {
  e.preventDefault();

  const room = STATE.room;
  const message = messageField.value;
  const username = usernameField.value || "guest";

  if (!message || !username) return;

  if (socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ room, username, message }));
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

async function sendAuthRequest(type) {
  const username = document.getElementById("auth-username").value;
  const password = document.getElementById("auth-password").value;
  const color = document.getElementById("auth-color").value;

  if (!username || !password) {
    authStatus.textContent = "Username and password required";
    return;
  }

  const body = new URLSearchParams();
  body.append("username", username);
  body.append("password", password);
  if (color) body.append("color", color);

  const response = await fetch(`/${type}`, {
    method: "POST",
    body
  });

  const data = await response.json();
  authStatus.textContent = data.message;

  if (data.status.includes("succesfully logged in") || data.status === "ok") {
      STATE.username = username;
      STATE.color = color || "white";   // ⭐ store chosen color
      document.getElementById("chat").style.display = "flex";
      document.getElementById("auth").style.display = "none";
    
      // window.shouldReconnect = true;
      await uploadHistory();
      connectWebSocket();
  }
}

function setupAuth() {
  const authForm = document.getElementById("auth-form");
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
  // if (window.shouldReconnect === false) {
  //   console.log("Reconnect disabled (logged out)");
  //   return;
  // }
  if (!STATE.username) {
    console.log("Not logged in, WS disabled");
    return;
  }

  let ws = new WebSocket("ws://localhost:8080/ws");

  ws.onopen = () => {
    setConnectedStatus(true);
    console.log("WebSocket connected");
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

  socket.onmessage = (event) => {
    const msg = JSON.parse(event.data);
  
    if (!msg.room || !msg.username || !msg.message) return;
  
    addMessage(msg.room, msg.username, msg.message, true);
  };

  window.chatSocket = ws;
}

// Let's go! Initialize the world.
function init() {

  // Initialize some rooms.
  addRoom("lobby");
  addRoom("rocket");
  changeRoom("lobby");
  addMessage("lobby", "Rocket", "Hey! Open another browser tab, send a message.", true);
  addMessage("rocket", "Rocket", "This is another room. Neat, huh?", true);

  
}

setupAuth();
init();
