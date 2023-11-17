// npx http-server -p 3001

const webSocket = new WebSocket('ws://localhost:7878/ws'); 

let iceServersconfig = {
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

let localStream;
let peerConn;
let roomId;
let isAudio = true;
let isVideo = true;

webSocket.onmessage = (event) => {
    handleSignallingData(JSON.parse(event.data));
};

const handleSignallingData = (data) => {
    switch (data.data_type) {
        case 'offer':
            peerConn.setRemoteDescription(data.offer);
            createAndSendAnswer();
            break
        case 'candidate':
            peerConn.addIceCandidate(data);
            console.log('data.candidate: ', data.candidate);
    };
};

const joinCall = () => {
    roomId = document.getElementById('room-id-input').value;
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

        peerConn = new RTCPeerConnection(iceServersconfig);

        for (const track of stream.getTracks()) {
            peerConn.addTrack(track, stream);
        };

        peerConn.ontrack = (e) => {
            document.getElementById('remote-video').srcObject = e.streams[0];
        };

        peerConn.onicecandidate = ((e) => {
            if (e.candidate == null) return
            sendRoomData({
                data_type: 'send_candidate',
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

        sendRoomData({
            data_type: 'join_call',
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
    }).catch((err) => {
        console.log(err);
    });
};

const createAndSendAnswer = () => {
    peerConn.createAnswer().then((answer) => {
        peerConn.setLocalDescription(answer);
        sendRoomData({
            data_type: 'send_answer',
            offer: {
                type: '',
                sdp: ''
            },
            answer: answer,
            candidate: {
                candidate: '', 
                sdpMid: '', 
                sdpMLineIndex: 0, 
                usernameFragment: '',
            }
        });
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