#!/usr/bin/env bash

squads-multisig-client \
    --program-id GQGNGBWyWLQJHnnpxpNjd4qwqRK17Z3V6APS6ALee6KD \
    program-config-init \
    --initializer-keypair config_initializer.json \
    --program-config-authority 2v4iR9uBFCkQcLtuRt4vh3qdooSJ4zAaLKrri6899Phx \
    --treasury 6hHpKqBLA4HigNzcZNDCoC9hDq4gnxcBCfU8rY9PhsfT \
    --multisig-creation-fee 10

squads-multisig-client \
    --program-id GQGNGBWyWLQJHnnpxpNjd4qwqRK17Z3V6APS6ALee6KD \
    multisig-create \
    --keypair config_initializer.json \
    --config-authority 5HTfiCkTRDJFHbuGS8b8hhYe8XMNv52ayKsh8TQRjZf1 \
    --threshold 2 \
    -m FV6iUdVw1gosWFFaqiq8iiUcqkHKZhfMFdMvhajtoxp6,7 \
    -m 3hUz6ELX8ea39GQHZBgeAhBK3rvQGwqA1x3T5sxkr4FY,7 \
    -m F3ZuXhjhNZR7x2VX74PM5gt9VczkmfmUhrxgNJd6dJNw,7 \
    --multisig-keypair multisig_keypair.json \
    --rent-collector ArP4upfVvZMi9t6AKN3QmScaKyjChuwx5fJGgZqBDxps
