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
RECIPIENT_PUB = os.environ.get("ECC_PUB_KEY", "ecc_public.pem")
SENDER_PRIV = os.environ.get("SENDER_PRIV_KEY", "sender_private.pem")


def ensure_sender_keys():
    if not os.path.exists(SENDER_PRIV):
        key = ecc.generate_keypair()
        ecc.save_private(key, SENDER_PRIV)
        ecc.save_public(key, SENDER_PRIV.replace("_private", "_public"))


def main():
    recipient_pub = ecc.load_public(RECIPIENT_PUB)
    sender_priv = ecc.load_private(SENDER_PRIV)

    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    mqtt_conn.configure(client)
    mqtt_conn.connect(client, keepalive=60)
    client.loop_start()

    signal = {"id": int(time.time()), "cmd": "start", "ts": int(time.time())}
    signal_json = json.dumps(signal).encode()
    signature = ecc.sign(sender_priv, signal_json)
    envelope = ecc.pack_envelope(signal_json, signature)
    blob = ecc.encrypt(recipient_pub, envelope)
    client.publish(TOPIC, base64.b64encode(blob), qos=1)
    print("sent signed + ECC-encrypted signal")
    time.sleep(1)
    client.loop_stop()


if __name__ == "__main__":
    ensure_sender_keys()
    main()
