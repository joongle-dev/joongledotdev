const join_button = document.getElementById('join-button');
const create_button = document.getElementById('create-button');
const search_params = new URLSearchParams(window.location.search);
const room_id = search_params.get('room');
if (room_id) {
    join_button.disabled = false;
    join_button.addEventListener('click', () => {
        const socket = new WebSocket('wss://www.joongle.dev/yahtzee1/ws?room=' + room_id);
        socket.binaryType = 'arraybuffer';
        socket.onopen = () => {
            console.log('open event');
        }
        socket.onclose = (event) => {
            switch (event.reason) {
                case 'disconnect':
                    break;
                case 'not found':
                    break;
                case 'full':
                    break;
                default:
            }
            console.log("close event")
        }
    });
}
create_button.addEventListener('click', () => {
    const socket = new WebSocket('wss://www.joongle.dev/yahtzee1/ws');
    socket.binaryType = 'arraybuffer';
    socket.onopen = () => {
        console.log('open event');
    }
    socket.onclose = (event) => {
        switch (event.reason) {
            case 'disconnect':
                break;
            case 'not found':
                break;
            case 'full':
                break;
            default:
        }
        console.log("close event")
    }
});