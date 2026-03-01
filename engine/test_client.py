import asyncio
import websockets
import json

async def test_chat():
    uri = "ws://localhost:8000/ws/rpc"
    async with websockets.connect(uri) as websocket:
        print(f"Connected to {uri}")
        
        # 1. Expect Handshake
        handshake = await websocket.recv()
        print(f"Handshake: {json.loads(handshake)}")
        
        # 2. Expect Sync State
        sync_state = await websocket.recv()
        print(f"Sync State: {json.loads(sync_state)}")
        
        # 3. Send Prompt
        prompt = {"prompt": "Who are you?"}
        await websocket.send(json.dumps(prompt))
        print("Sent prompt")
        
        # 4. Expect Thinking
        thinking = await websocket.recv()
        print(f"Thinking: {json.loads(thinking)}")
        
        # 5. Expect stream and chat_end
        full_msg = ""
        while True:
            chunk = await websocket.recv()
            data = json.loads(chunk)
            
            if data["event"] == "chat_end":
                print(f"\n[Finished Streams. Total tokens: {data['payload']['metadata'].get('total_tokens')}]")
                break
            elif data["event"] == "chat_chunk":
                content = data["payload"]["content"]
                metadata = data["payload"]["metadata"]
                full_msg += content
                print(content, end="", flush=True)
                # optionally print tps
                # print(f" (TPS: {metadata.get('tokens_per_second')})", end="", flush=True)

if __name__ == "__main__":
    asyncio.run(test_chat())
