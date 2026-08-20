import asyncio
import os

HOST = os.environ.get("MQTT_HOST", "127.0.0.1")
PORT = int(os.environ.get("MQTT_PORT", "1883"))

subscribers = set()


def encode_remaining(length):
    out = []
    while True:
        b = length % 128
        length //= 128
        if length > 0:
            b |= 0x80
        out.append(b)
        if length == 0:
            break
    return bytes(out)


def decode_remaining(data, idx):
    multiplier = 1
    value = 0
    while True:
        b = data[idx]
        idx += 1
        value += (b & 0x7F) * multiplier
        if (b & 0x80) == 0:
            break
        multiplier *= 128
    return value, idx


def make_connack():
    return bytes([0x20, 0x02, 0x00, 0x00])


def make_suback(packet_id, count):
    return bytes([0x90]) + encode_remaining(2 + count) + packet_id.to_bytes(2, "big") + bytes([0x00] * count)


def make_pingresp():
    return bytes([0xD0, 0x00])


def make_publish(topic, payload):
    topic_bytes = topic.encode()
    body = len(topic_bytes).to_bytes(2, "big") + topic_bytes + payload
    return bytes([0x30]) + encode_remaining(len(body)) + body


async def handle_client(reader, writer):
    subscribers.add(writer)
    buffer = b""
    try:
        while True:
            data = await reader.read(4096)
            if not data:
                break
            buffer += data
            while len(buffer) >= 2:
                packet_type = buffer[0] >> 4
                remaining, idx = decode_remaining(buffer, 1)
                if len(buffer) < idx + remaining:
                    break
                payload = buffer[idx:idx + remaining]
                buffer = buffer[idx + remaining:]

                if packet_type == 1:
                    writer.write(make_connack())
                elif packet_type == 8:
                    packet_id = int.from_bytes(payload[:2], "big")
                    count = max(1, (len(payload) - 2) // 3)
                    writer.write(make_suback(packet_id, count))
                elif packet_type == 3:
                    tlen = int.from_bytes(payload[:2], "big")
                    topic = payload[2:2 + tlen].decode()
                    msg = payload[2 + tlen:]
                    for sub in subscribers:
                        if sub is not writer:
                            try:
                                sub.write(make_publish(topic, msg))
                            except Exception:
                                pass
                elif packet_type == 12:
                    writer.write(make_pingresp())
                elif packet_type == 14:
                    break
                await writer.drain()
    except asyncio.CancelledError:
        pass
    finally:
        subscribers.discard(writer)
        try:
            writer.close()
        except Exception:
            pass


async def main():
    server = await asyncio.start_server(handle_client, HOST, PORT)
    print(f"minimal MQTT broker on {HOST}:{PORT}")
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(main())
