import json
import base64
import os
import time

import paho.mqtt.client as mqtt

import ecc_crypto as ecc
import mqtt_conn

BROKER_HOST = os.environ.get("MQTT_HOST", "localhost")
BROKER_PORT = int(os.environ.get("MQTT_PORT", "1883"))
TOPIC = os.environ.get("MQTT_TOPIC", "signals/encrypted")
PRIV_KEY_FILE = os.environ.get("ECC_PRIV_KEY", "ecc_private.pem")
PUB_KEY_FILE = os.environ.get("ECC_PUB_KEY", "ecc_public.pem")
SENDER_PUB_FILE = os.environ.get("SENDER_PUB_KEY", "sender_public.pem")

MAX_CLOCK_SKEW = 60
PROCESSED = set()
PROCESSED_MAX = 10000


def load_or_create_keys():
    if not os.path.exists(PRIV_KEY_FILE):
        key = ecc.generate_keypair()
        ecc.save_private(key, PRIV_KEY_FILE)
        ecc.save_public(key, PUB_KEY_FILE)
        print(f"generated receiver keypair -> {PRIV_KEY_FILE}, {PUB_KEY_FILE}")
    if not os.path.exists(SENDER_PUB_FILE):
        key = ecc.generate_keypair()
        ecc.save_private(key, SENDER_PUB_FILE.replace("_public", "_private"))
        ecc.save_public(key, SENDER_PUB_FILE)
        print(f"generated sender keypair (demo) -> {SENDER_PUB_FILE}")


def is_fresh(signal):
    sid = signal.get("id")
    ts = signal.get("ts")
    now = int(time.time())
    if sid is None or ts is None:
        return False
    if abs(now - ts) > MAX_CLOCK_SKEW:
        return False
    if sid in PROCESSED:
        return False
    PROCESSED.add(sid)
    if len(PROCESSED) > PROCESSED_MAX:
        PROCESSED.clear()
    return True


def on_connect(client, userdata, flags, rc, properties=None):
    print(f"connected to broker rc={rc}")
    client.subscribe(TOPIC, qos=1)


def on_message(client, userdata, msg):
    priv = userdata["priv"]
    sender_pub = userdata["sender_pub"]
    try:
        blob = base64.b64decode(msg.payload)
        envelope = ecc.decrypt(priv, blob)
        signal_json, signature = ecc.unpack_envelope(envelope)
        if not ecc.verify(sender_pub, signature, signal_json):
            print("REJECTED: invalid signature")
            return
        signal = json.loads(signal_json)
        if not is_fresh(signal):
            print(f"REJECTED: replay/stale signal id={signal.get('id')}")
            return
        print(f"received verified signal: {signal}")
    except Exception as e:
        print(f"decryption/processing failed: {e}")


def main():
    load_or_create_keys()
    priv = ecc.load_private(PRIV_KEY_FILE)
    sender_pub = ecc.load_public(SENDER_PUB_FILE)
    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    client.user_data_set({"priv": priv, "sender_pub": sender_pub})
    mqtt_conn.configure(client)
    mqtt_conn.connect(client, keepalive=60)
    client.loop_forever()


if __name__ == "__main__":
    main()
