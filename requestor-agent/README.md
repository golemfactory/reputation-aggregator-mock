#   Alpha 1 requestor agent

##  General purpose

Run some useless tasks on the Golem network.

Key elements:

*   We pay only for tasks correctly executed
*   We use the global A1 reputation to filter out the low-performing providers
*   Task weight varies between runs

##  Usage

```
python3 requestor.py 
    #   Important required args
    --repu-factor   INT  # 0-100 the higher the factor the higher is the chance we'll refuse the offer
                         # because of the reputation.
                         #   0    -> we don't care about the reputation
                         #   100  -> we accept offers only from the single best provider (among current offers)
                         # NOTE: we'll always accept offers from providers without known reputation
    --min-offers    INT  # How many offers we should gather before we start selecting providers
    --task-size     INT  # Higher number = longer task. On the devnet-beta this is more-or-less
                         # the number of seconds required to perform the computations.
                         # NOTE:  This also influences the task timeout. Timeout is set to 5 * task_size.
                         #        (this doesn't cover the time required to download the image)
                         # NOTE2: Currently there is some limit on the task-size because task data is sent in
                         #        a command, not a file. This can be easily fixed.
    --num-providers INT  # How many different providers will be tested.
                         # We'll always run only a single task per provider.


    #   Common yapapi args that have defaults & work just as in any yapapi example
    --subnet-tag
    --payment-network
    --payment-driver
    --log-file
```

##  Requirements

`yapapi` - latest `master` should be fine, developed & tested with `53247a6` (10.05.2022).

```
pip3 install git+https://github.com/golemfactory/yapapi.git@jb/proposal-received-scored-split
pip3 install primefac==2.0.12
```

(Note: `yapapi==0.9.*` is **not** enough).


