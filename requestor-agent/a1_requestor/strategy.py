import asyncio
from typing import Union
from decimal import Decimal

from yapapi.strategy import MarketStrategy
from yapapi.events import ProposalReceived, TaskAccepted

from .worker import TaskTimeout, IncorrectResult


class AlphaRequestorStrategy(MarketStrategy):
    def __init__(self, *, min_offers, repu_factor):
        self.min_offers = min_offers
        self.repu_factor = repu_factor

        self._offers = set()
        self._failed_activities = set()
        self._payable_agreements = set()

    async def debit_note_accepted_amount(self, debit_note):
        if debit_note.activity_id in self._failed_activities:
            return Decimal(0)
        return Decimal(debit_note.total_amount_due)

    async def invoice_accepted_amount(self, invoice):
        if invoice.agreement_id not in self._payable_agreements:
            return Decimal(0)
        return Decimal(invoice.amount)

    async def score_offer(self, offer):
        while len(self._offers) < self.min_offers:
            print(f"Waiting for {self.min_offers} offers, current count: {len(self._offers)})")
            await asyncio.sleep(1)

        if self._price_too_high(offer):
            return -1
        return await self._reputation_score(offer.issuer)

    def event_consumer(self, event: Union[ProposalReceived, TaskTimeout, IncorrectResult, TaskAccepted]) -> None:
        if isinstance(event, ProposalReceived):
            self._offers.add(event.proposal)
        elif isinstance(event, TaskAccepted):
            agreement_id = event.agreement.id
            self._payable_agreements.add(agreement_id)
        else:
            activity_id = event.activity.id
            self._failed_activities.add(activity_id)

    def _price_too_high(self, offer) -> bool:
        #   TODO: set a limit here, we don't want to pay too much
        return False

    async def _reputation_score(self, provider_id) -> float:
        #   TODO:
        #   1.  Get the reputation for the provider
        #   2.  If it is empty, return 1
        #   3.  If not, sometimes return -1 (the more often the higher is self.repu_factor
        #       and the lower is reputation), otherwise 1.
        return 1
