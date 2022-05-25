import asyncio
from datetime import datetime, timedelta
from typing import Optional, Union
from decimal import Decimal

from yapapi.strategy import MarketStrategy, LeastExpensiveLinearPayuMS
from yapapi.events import ProposalReceived, TaskAccepted
from yapapi.props import com

from .worker import TaskTimeout, IncorrectResult


class AlphaRequestorStrategy(MarketStrategy):
    def __init__(self, *, min_offers, repu_factor, wait_for_offers_timeout_seconds):
        self.min_offers = min_offers
        self.repu_factor = repu_factor
        self.wait_for_offers_timeout = timedelta(seconds=wait_for_offers_timeout_seconds)

        self._offers = set()
        self._failed_activities = set()
        self._payable_agreements = set()
        self._scored_providers = set()

        self._wait_for_offers_task: Optional[asyncio.Task] = None

    async def debit_note_accepted_amount(self, debit_note):
        if debit_note.activity_id in self._failed_activities:
            return Decimal(0)
        return Decimal(debit_note.total_amount_due)

    async def invoice_accepted_amount(self, invoice):
        if invoice.agreement_id not in self._payable_agreements:
            return Decimal(0)
        return Decimal(invoice.amount)

    async def score_offer(self, offer):
        if self._wait_for_offers_task is None:
            self._wait_for_offers_task = asyncio.create_task(self._wait_for_offers())
        await asyncio.wait_for(self._wait_for_offers_task, None)

        #   Ensure we have no more than a single agreement with the same provider
        #   NOTE: This also prevents us from creating multiple agreements from the same offer
        #         because offer is rescored after agreement ended.
        #         (I tried to find this logic in yapapi and failed, but seems to work).
        #   Also: offer.is_draft is necessary because there is a non-draft and later draft offer
        #         before the agreement is signed, and we don't want to reject the latter.
        if not offer.is_draft and offer.issuer in self._scored_providers:
            return -1
        self._scored_providers.add(offer.issuer)

        if await self._price_too_high(offer):
            provider_name = offer._proposal.proposal.properties['golem.node.id.name']
            print(f"Rejected offer from {provider_name} ({offer.issuer}) because of the price")
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

    async def _price_too_high(self, offer) -> bool:
        #   NOTE: we use this strategy because it's easier than extracting important
        #         logic from it and writing the logic directly.
        lelp = LeastExpensiveLinearPayuMS(
            max_fixed_price=Decimal(0.1),
            max_price_for={
                com.Counter.CPU: Decimal(0.1),
                com.Counter.TIME: Decimal(0.1),
            },
        )
        lelp_score = await lelp.score_offer(offer)
        return lelp_score < 0

    async def _reputation_score(self, provider_id) -> float:
        #   TODO:
        #   1.  Get the reputation for the provider
        #   2.  If it is empty, return 1
        #   3.  If not, sometimes return -1 (the more often the higher is self.repu_factor
        #       and the lower is reputation), otherwise 1.
        return 1

    async def _wait_for_offers(self) -> None:
        deadline = datetime.now() + self.wait_for_offers_timeout
        while len(self._offers) < self.min_offers and datetime.now() < deadline:
            print(
                f"Waiting for either {self.min_offers} offers or for {(deadline - datetime.now()).seconds} "
                f"seconds more, current offers count: {len(self._offers)}"
            )
            await asyncio.sleep(1)
