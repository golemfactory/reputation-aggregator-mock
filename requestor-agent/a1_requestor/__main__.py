#!/usr/bin/env python3
from datetime import datetime
import pathlib
import sys

from yapapi import (
    Golem,
    Task,
)
from yapapi.payload import vm

examples_dir = pathlib.Path(__file__).resolve().parent.parent
sys.path.append(str(examples_dir))

from a1_requestor.yapapi_example_utils import (
    build_parser,
    run_golem_example,
    print_env_info,
)
from a1_requestor.worker import worker, IMAGE_HASH, prepare_task_data, TaskTimeout, IncorrectResult
from a1_requestor.strategy import AlphaRequestorStrategy


async def main(golem, *, num_providers, task_size, random_fail_factor):
    payload = await vm.repo(image_hash=IMAGE_HASH)

    async with golem:
        print_env_info(golem)

        task_data = prepare_task_data(task_size)
        tasks = [Task(data=(task_data, random_fail_factor)) for _ in range(num_providers)]

        num_tasks = 0
        async for _ in golem.execute_tasks(worker, tasks, payload, max_workers=num_providers):
            num_tasks += 1
            print(f"{num_tasks} out of {num_providers} planned providers tested")


if __name__ == "__main__":
    parser = build_parser("Run an alpha1 requestor")
    now = datetime.now().strftime("%Y-%m-%d_%H.%M.%S")
    parser.set_defaults(log_file=f"a1_requestor-{now}.log")

    parser.add_argument("--repu-factor", type=int, required=True)
    parser.add_argument("--min-offers", type=int, required=True)
    parser.add_argument("--task-size", type=int, required=True)
    parser.add_argument("--num-providers", type=int, required=True)
    parser.add_argument("--offers-wait-timeout", type=int, default=60)
    parser.add_argument("--random-fail-factor", type=int, default=0)

    args = parser.parse_args()

    strategy = AlphaRequestorStrategy(
        min_offers=args.min_offers,
        repu_factor=args.repu_factor,
        wait_for_offers_timeout_seconds=args.offers_wait_timeout,
    )
    golem = Golem(
        budget=10,  # TODO: do we need to parametrize this?
        strategy=strategy,
        subnet_tag=args.subnet_tag,
        payment_driver=args.payment_driver,
        payment_network=args.payment_network,
    )
    golem.add_event_consumer(
        strategy.event_consumer,
        ["ProposalReceived", TaskTimeout, IncorrectResult, "TaskAccepted", "AgreementRejected"]
    )

    run_golem_example(
        main(
            golem,
            num_providers=args.num_providers,
            task_size=args.task_size,
            random_fail_factor=args.random_fail_factor,
        ),
        log_file=args.log_file,
    )

    print(f"PAYABLE AGREEMENTS: {len(strategy._payable_agreements)}")
    print(f"FAILED ACTIVITIES: {len(strategy._failed_activities)}")
