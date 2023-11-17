// npx http-server -p 3000


const webSocket = new WebSocket('ws://localhost:7878/ws');

const iceServersConfig = {
    iceServers: [
        {
            urls: [
                'stun:stun.l.google.com:19302', 
                'stun:stun1.l.google.com:19302', 
                'stun:stun2.l.google.com:19302'
            ]
        }
    ]
};

let roomId;
let localStream;
let peerConn;
let isAudio = true;
let isVideo = true;

webSocket.onmessage = (event) => {
    handleSignallingData(JSON.parse(event.data));
};

const handleSignallingData = (data) => {
    switch (data.data_type) {
        case 'answer':
            peerConn.setRemoteDescription(data.answer);
            break;
        case 'candidate':
            peerConn.addIceCandidate(data.candidate);
            console.log('data.candidate: ', data.candidate);
            webSocket.onerror = console.log;
    };
};

const sendRoomId = () => {
    roomId = document.getElementById('room-id-input').value;
    sendRoomData({
        data_type: 'store_room',
        offer: {
            type: '',
            sdp: ''
        },
        answer: {
            type: '',
            sdp: ''
        },
        candidate: {
            candidate: '', 
            sdpMid: '', 
            sdpMLineIndex: 0, 
            usernameFragment: '',
        }
    });
};

const startCall = () => {
    document.getElementById('video-call-div').style.display = 'inline';

    navigator.mediaDevices.getUserMedia({
        video: {
            frameRate: 24,
            width: { min: 480, ideal: 720, max: 1280 },
            aspectRatio: 1.33333
        },
        audio: true
    }).then((stream) => {
        localStream = stream;
        document.getElementById('local-video').srcObject = localStream;

        peerConn = new RTCPeerConnection(iceServersConfig);

        for (const track of stream.getTracks()) {
            peerConn.addTrack(track, stream);
        };

        peerConn.ontrack = (e) => {
            document.getElementById('remote-video').srcObject = e.streams[0];
            console.log('e.streams: ', e.streams);
            console.log('e.streams[0]: ', e.streams[0]);
        };

        peerConn.onicecandidate = ((e) => {
            if (e.candidate == null) return
            sendRoomData({
                data_type: 'store_candidate',
                offer: {
                    type: '',
                    sdp: ''
                },
                answer: {
                    type: '',
                    sdp: ''
                },
                candidate: e.candidate
            });
        });
        createAndSendOffer();
    }).catch((err) => {
        console.log(err);
    });
};

const createAndSendOffer = () => {
    peerConn.createOffer().then((offer) => {
        sendRoomData({
            data_type: 'store_offer',
            offer: offer,
            answer: {
                type: '',
                sdp: ''
            },
            candidate: {
                candidate: '', 
                sdpMid: '', 
                sdpMLineIndex: 0, 
                usernameFragment: '',
            }
        });
        peerConn.setLocalDescription(offer);
    }).catch((err) => {
        console.log(err);
    });
};

const sendRoomData = (data) => {
    data.room_id = roomId;
    webSocket.send(JSON.stringify(data));
};

const muteAudio = () => {
    isAudio = !isAudio;
    localStream.getAudioTracks()[0].enabled = isAudio;
};

const muteVideo = () => {
    isVideo = !isVideo;
    localStream.getVideoTracks()[0].enabled = isVideo;
};