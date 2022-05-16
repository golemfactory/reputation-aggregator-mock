from yapapi.strategy import MarketStrategy


class AlphaRequestorStrategy(MarketStrategy):
    def __init__(self, min_offers, repu_factor):
        self.min_offers = min_offers
        self.repu_factor = repu_factor

    async def score_offer(self, offer):
        return 7
