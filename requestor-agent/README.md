#   Alpha 1 requestor agent

##  General purpose

Run some useless tasks on the Golem network.

Key elements:

*   We pay only for tasks correctly executed
*   We use the global A1 reputation to filter out the low-performing providers
*   Task weight varies between runs

##  Installation

```
pip3 install git+https://github.com/golemfactory/reputation-aggregator-mock.git@jb/a1-requestor-agent#subdirectory=requestor-agent
```

##  Usage

```
python3 -m a1_requestor
    #   Important required args
    --repu-factor   INT  # 0-100 the higher the factor the higher is the chance we'll refuse the offer
                         # because of the reputation.
                         #   0    -> we don't care about the reputation
                         #   100  -> we accept offers only from the single best provider (among current offers)
                         # NOTE: we'll always accept offers from providers without known reputation
    --min-offers    INT  # How many offers we should gather before we start selecting providers 
                         # (check also offers_wait_timeout argument)
    --task-size     INT  # Higher number = longer task. On the devnet-beta this is more-or-less
                         # the number of seconds required to perform the computations.
                         # NOTE:  This also influences the task timeout. Timeout is set to 5 * task_size.
                         #        (this doesn't cover the time required to download the image)
                         # NOTE2: Currently there is some limit on the task-size (~400, exact number may vary between runs)
                         #        because task data is sent in a command, not a file. This can be easily fixed.
    --num-providers INT  # End execution after this number of **successful** task runs.
                         # We'll always run only a single task per provider.

    #   Optional 
    --offers-wait-timeout INT  # After this time (seconds) stop waiting for min-offers even if we didn't reach the treshold
                               # Defaults to 60.
    --random-fail-factor  INT  # 0-100 % of tasks that will fail without any fault on the provider side)
                               # (this is intended strictly for development/testing), defaults to 0

    #   Common yapapi args that have defaults & work just as in any yapapi example
    --subnet-tag
    --payment-network
    --payment-driver
    --log-file
```
