const canvas = document.getElementById('canvas');

const joining = document.getElementById('joining');
const failed = document.getElementById('failed');
const joined = document.getElementById('joined');

const socket = new WebSocket("wss://" + window.location.host + window.location.pathname + "ws" + window.location.search);
socket.binaryType = "arraybuffer";
socket.onopen = () => {
    socket.onmessage = (ev) => {
        //const data = new Uint8Array(ev.data);
        console.log("message received", ev.data);
    }
}
socket.onclose = () => {
    console.log("websocket close");
}
socket.onerror = (ev) => {
    console.log("websocket error: ", ev);
}