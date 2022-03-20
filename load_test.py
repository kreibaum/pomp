# Python utility to connet to the server and run load tests
# Multiple connections are opened and on each connection we guess
# an answer every few seconds.
# We should then see how the host view and projector view behave and if it
# can scale to this size.

# Here is how you manually connect to the server with a console:
# python -m websockets "ws://0.0.0.0:8080/ws?uuid=d3fdf683-9fa0-4d90-b93b-0ecec8c9d96c"

import asyncio
import websockets
import uuid
import random

N_CONNECTIONS = 200


# Generate a random name
def random_name():
    return "".join(random.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(5))


async def connect():
    async with websockets.connect(f"ws://localhost:8080/ws?uuid={uuid.uuid4()}") as websocket:
        await asyncio.gather(producer(websocket), consumer(websocket))


async def producer(websocket):
    """First send a name, then send guesses in an infinite loop with a random wait time"""
    name = random_name()
    await websocket.send(f"{{\"SetName\":\"{name}\"}}")
    while True:
        # Either guess "Bride" or "Groom" with 50% chance
        await websocket.send(f"{{\"SetGuess\":\"{random.choice(['Bride', 'Groom'])}\"}}")
        # Sleep a random amount of time between 1 and 3 seconds
        await asyncio.sleep(1 + (2 * random.random()))


async def consumer(websocket):
    """Receive messages from the server to keep the queue empty."""
    while True:
        message = await websocket.recv()

# asyncio.run(connect())
# Runn N_CONNECTIONS connections at the same time with asyncio
loop = asyncio.get_event_loop()
loop.run_until_complete(asyncio.gather(
    *[connect() for i in range(N_CONNECTIONS)]))
loop.close()
