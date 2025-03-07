const join_button = document.getElementById('join-button');
const create_button = document.getElementById('create-button');
const search_params = new URLSearchParams(window.location.search);
const room_id = search_params.get('room');
if (room_id) {
    join_button.disabled = false;
    join_button.addEventListener('click', () => {
        const socket = new WebSocket('wss://joongle.dev/yahtzee1/ws?room=' + room_id);
        socket.binaryType = 'arraybuffer';
        socket.onopen = () => {
            socket.onmessage = (event) => {
                console.log('message event: ' + event.data)
            }
            console.log('open event');
        }
        socket.onclose = (event) => {
            console.log('close event: ', event.code, ': ',event.reason);
        }
        socket.onerror = (event) => {
            console.log('error event: ' + event);
        }
    });
}
create_button.addEventListener('click', () => {
    const socket = new WebSocket('wss://joongle.dev/yahtzee1/ws');
    socket.binaryType = 'arraybuffer';
    socket.onopen = () => {
        socket.onmessage = (event) => {
            console.log('message event: ' + event.data)
        }
        console.log('open event');
    }
    socket.onclose = (event) => {
        console.log('close event: ', event.code, ': ',event.reason);
    }
    socket.onerror = (event) => {
        console.log('error event: ' + event);
    }
});
