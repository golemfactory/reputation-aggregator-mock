import aiohttp
import asyncio
from datetime import datetime, timedelta
from typing import Optional, Union
from decimal import Decimal
from random import random

from yapapi.strategy import MarketStrategy, LeastExpensiveLinearPayuMS
from yapapi.events import ProposalReceived, TaskAccepted
from yapapi.props import com

from .worker import TaskTimeout, IncorrectResult

PROVIDER_STANDARD_SCORE_URL = "http://reputation.dev.golem.network/standard_score/provider/{}"


async def get_provider_standard_score(provider_id: str) -> Optional[float]:
    url = PROVIDER_STANDARD_SCORE_URL.format(provider_id)

    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as response:
                if response.status != 200:
                    return None
                score_str = (await response.json())['score']
                score = float(score_str) if score_str is not None else None
                return score
    except aiohttp.client_exceptions.ClientError:
        print("Reputation service is down")
        return None


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

        self._provider_scores = {}

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
            if event.proposal.issuer not in self._provider_scores:
                provider_name = event.proposal._proposal.proposal.properties['golem.node.id.name']
                asyncio.create_task(self._load_score(event.proposal.issuer, provider_name))

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
        while provider_id not in self._provider_scores:
            #   There is a separate task that is getting the score now
            await asyncio.sleep(1)

        score = self._provider_scores[provider_id]
        if score is None:
            #   New provider, we want to try them
            return 1

        better_cnt = sum(1 for _, other_score in self._provider_scores.items() if other_score > score)

        #   Compute the probability that offer will be accepted. This is e.g.:
        #       1 if self._repu_factor is 0
        #       1 if this is the best provider
        #       Almost 0 if self._repu_factor is 100 and this is the worst provider
        #       Around 0.25 if self._repu_factor is 100 and this is the median provider
        accepted_prob = 1 - (self.repu_factor / 100) * ((better_cnt / len(self._provider_scores))**2)

        if random() < accepted_prob:
            #   This offer is accepted and we no longer care about reputation,
            #   but we want the (accepted) score to be randomized because this way
            #   we don't care (or maybe care less) about the incoming offers order
            print(f"Accepted offer with score {score}")
            return 1 - random()
        else:
            print(f"Refused offer with score {score} because there are {better_cnt} better providers")
            return -1

    async def _wait_for_offers(self) -> None:
        deadline = datetime.now() + self.wait_for_offers_timeout
        while len(self._offers) < self.min_offers and datetime.now() < deadline:
            print(
                f"Waiting for either {self.min_offers} offers or for {(deadline - datetime.now()).seconds} "
                f"seconds more, current offers count: {len(self._offers)}"
            )
            await asyncio.sleep(1)

    async def _load_score(self, provider_id, provider_name):
        score = await get_provider_standard_score(provider_id)
        print(f"Score for provider {provider_name} ({provider_id}) is {score}")
        self._provider_scores[provider_id] = score
