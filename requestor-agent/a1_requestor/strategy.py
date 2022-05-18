import asyncio
from typing import Union

from yapapi.strategy import MarketStrategy
from yapapi.events import ProposalReceived, TaskAccepted

from .worker import TaskTimeout, IncorrectResult


class AlphaRequestorStrategy(MarketStrategy):
    def __init__(self, *, min_offers, repu_factor):
        self.min_offers = min_offers
        self.repu_factor = repu_factor

        self._offers = set()
        self._failed_activities = set()
        self._payable_activities = set()

    async def score_offer(self, offer):
        while len(self._offers) < self.min_offers:
            print(f"Waiting for {self.min_offers} offers, current count: {len(self._offers)})")
            await asyncio.sleep(1)

        return 7

    def event_consumer(self, event: Union[ProposalReceived, TaskTimeout, IncorrectResult, TaskAccepted]) -> None:
        if isinstance(event, ProposalReceived):
            self._offers.add(event.proposal)
        else:
            activity_id = event.activity.id
            if isinstance(event, TaskAccepted):
                self._payable_activities.add(activity_id)
            else:
                self._failed_activities.add(activity_id)
