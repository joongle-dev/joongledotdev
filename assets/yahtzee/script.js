//import init, { run } from './yahtzee.js';
//
// const canvas = document.getElementById('canvas');
//
// async function run_yahtzee() {
//     await init();
//     await run(canvas);
// }
// await run_yahtzee();

import init, { deserialize_message, serialize_message } from './yahtzee.js';

await init();

const name_input = document.getElementById('name-input');
const join_lobby_btn = document.getElementById('name-submit-btn');
const ping_btn = document.getElementById('ping-btn');

const peer_map = new Map();
const websocket_protocol = location.protocol !== 'https:' ? 'ws:' : 'wss:';
const websocket_address = websocket_protocol + '//' + location.host + location.pathname + 'ws' + location.search;
let websocket = null;
let lobby_id = 0;
let user_id = 0;
let username = '';

const configuration = {
    iceServers: [
        {
            urls: 'turn:turn.joongle.dev:3478',
            username: 'guest',
            credential: 'guest1234',
        },
        {
            urls: 'turn:turn.joongle.dev:5349',
            username: 'guest',
            credential: 'guest1234',
        },
    ],
};

function create_pc(peer_id) {
    const pc = new RTCPeerConnection(configuration);
    pc.onicecandidate = (event) => {
        console.log('Ice candidate event');
        const peer_ref = peer_map.get(peer_id);
        if (event.candidate !== null) {
            peer_ref.candidates.push(event.candidate);
        }
        else {
            console.log('Ice candidates gathered, sending sdp handshake...');
            const serialized = serialize_message(user_id, peer_ref.id, username, peer_ref.sdp, peer_ref.candidates);
            websocket.send(serialized.buffer);
        }
    };
    return pc;
}

function configure_data_channel(dc) {
    dc.onopen = (event) => {
        console.log('Data channel opened!');
    };
    dc.onclose = (event) => {
        console.log('Data channel closed!');
    }
    dc.onmessage = (event) => {
        console.log('Data channel message: ' + event.data);
    }
}
async function create_offer(peer_id) {
    console.log('Creating offer to Peer ID: ' + peer_id);
    const pc = create_pc(peer_id);
    const dc = pc.createDataChannel('Data channel');
    console.log('RTCPeerConnection created');
    pc.createOffer().then((offer) => {
        pc.setLocalDescription(offer).then(() => {
            peer_map.set(peer_id, { id: peer_id, pc: pc, dc: dc, sdp: offer.sdp, candidates: [] });
        })
    });
}

async function create_answer(peer_id, name, sdp, candidates) {
    console.log('Received sdp offer from ' + name);
    const pc = create_pc(peer_id);
    pc.setRemoteDescription({ sdp: sdp, type: 'offer' }).then(() => {
        pc.createAnswer().then((answer) => {
            pc.setLocalDescription(answer).then(() => {
                candidates.forEach((candidate) => {
                    pc.addIceCandidate(candidate);
                });
                peer_map.set(peer_id, { id: peer_id, name: name, pc: pc, sdp: answer.sdp, candidates: [] });
            })
        })
    });
}

async function receive_answer(peer_id, name, sdp, candidates) {
    console.log('Received sdp answer from ' + name);
    const peer_ref = peer_map.get(peer_id);
    const pc = peer_ref.pc;
    peer_ref.name = name;
    pc.setRemoteDescription({ sdp: sdp, type: 'answer' }).then(() => {
        candidates.forEach((candidate) => {
            pc.addIceCandidate(candidate);
        })
    });
}

join_lobby_btn.onclick = (_event) => {
    username = name_input.textContent;
    join_lobby_btn.hidden = true;
    ping_btn.hidden = false;

    websocket = new WebSocket(websocket_address);
    websocket.binaryType = 'arraybuffer';
    websocket.onmessage = (event) => {
        const message = deserialize_message(new Uint8Array(event.data));
        if (message.is_connect_success_message()) {
            lobby_id = message.lobby_id;
            user_id = message.assigned_id;
            console.log('Invite code to lobby: ' + websocket_protocol + '//' + location.host + location.pathname + '?lobby_id=' + lobby_id);
            console.log('Assigned ID: ' + user_id);
            message.peers_id.forEach((peer_id) => create_offer(peer_id));
        }
        else {
            if (peer_map.has(message.source)) {
                receive_answer(message.source, message.username, message.sdp_description, message.ice_candidates).then(() => {});
            }
            else {
                create_answer(message.source, message.username, message.sdp_description, message.ice_candidates).then(() => {});
            }
        }
    };
};

ping_btn.onclick = (_event) => {
    peer_map.forEach((peer_ref, peer_id, map) => {
        if (peer_ref.dc != null) {
            peer_ref.dc.send('Ping from ' + username);
        }
    });
};