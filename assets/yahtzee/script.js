const socket_address = "wss://" + window.location.host + "/yahtzee/ws";
console.log("connecting to " +  socket_address);
const socket = new WebSocket(socket_address);

socket.addEventListener('open', function(event) {
    console.log('websocket connection established');
});

socket.addEventListener('message', function(event) {
    console.log('websocket message received');
})