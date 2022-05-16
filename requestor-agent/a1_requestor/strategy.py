import asyncio

from yapapi.strategy import MarketStrategy
from yapapi.events import ProposalReceived


class AlphaRequestorStrategy(MarketStrategy):
    def __init__(self, *, min_offers, repu_factor):
        self.min_offers = min_offers
        self.repu_factor = repu_factor
        self._offers = set()

    async def score_offer(self, offer):
        while len(self._offers) < self.min_offers:
            print(f"Waiting for {self.min_offers} offers, current count: {len(self._offers)})")
            await asyncio.sleep(1)

        return 7

    def event_consumer(self, event: "ProposalReceived") -> None:
        self._offers.add(event.proposal)
