#!/bin/bash
set -e

# Stress test the eBPF network observability tool using iperf3

if ! command -v iperf3 &> /dev/null; then
    echo "iperf3 is not installed. Run scripts/lab-setup.sh first."
    exit 1
fi

PORT=5201
DURATION=30
PARALLEL_STREAMS=100

echo "[*] Starting iperf3 server on port $PORT in the background..."
iperf3 -s -p $PORT -D

echo "[*] Sleeping for 2 seconds to allow server to bind..."
sleep 2

echo "[*] Generating synthetic TCP flow load (Duration: ${DURATION}s, Streams: ${PARALLEL_STREAMS})..."
echo "[*] This will trigger ANOMALY_DATA_SPIKE events in the ring buffer."
iperf3 -c 127.0.0.1 -p $PORT -t $DURATION -P $PARALLEL_STREAMS

echo "[*] Stress test complete. Stopping iperf3 server..."
pkill iperf3

echo "[*] Done. Check the eBPF ringbuffer JSON output for anomaly alerts!"
