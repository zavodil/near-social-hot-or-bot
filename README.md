Game on top of near.social protocol
======

Game rules:
- Choose which photo is real (hot) and which is generated by AI (bot).
- Give 4 of 4 correct answers to receive an NFT.

How it works:
- Near Social widget loads challenges for a given account from this contract using free view calls.
- Widget stores data on near.social protocol
- After 4 turns user executes finalization method on this contract
  1) Contract reads game logs from near.social protocol
  2) Contract verifies if all challenges are valid
  3) Mint NFT for winner 
  4) Creates a badge on Near Social protocol for winner

Live Demo: 
https://near.social/#/zavodil.near/widget/hot-or-bot