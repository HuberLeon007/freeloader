export function frameMessage(message){const json=JSON.stringify(message);const bytes=new TextEncoder().encode(json);const frame=new Uint8Array(4+bytes.length);new DataView(frame.buffer).setUint32(0,bytes.length,true);frame.set(bytes,4);return frame;}
export function refusalMessage(){return {ok:false,error:'Cookies and credentials are never forwarded.'};}
