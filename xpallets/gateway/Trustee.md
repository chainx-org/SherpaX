# Overview

This document describes how to become a member of council and trustee. Becoming a trust, the main responsibility is to keep the btc for users and help with withdrawals. Every month the trust can apply to the treasury for `10500000/12*0.05=43750` ksx as rewards. Rewards are distributed in proportion to the number of withdrawals and the number of BTCs that help users withdraw.

# Council

## Elected candidate

![img](https://cdn.jsdelivr.net/gh/hacpy/PictureBed@master/Document/16399971419031639997141896.png)

To become a member of council, one must first be elected as a candidate.

## Vote

![img](https://cdn.jsdelivr.net/gh/hacpy/PictureBed@master/Document/16399970352971639997035289.png)

Everyone can stake some **ksx** and vote for multiple candidates. Allow yourself to vote for yourself. **After becoming a candidate, members of the parliament will be updated every day, and the ranking will be calculated based on the number of votes and related staking ksx.**

# Trustee

To become a trust, you must first be elected as a council member or runners up. **Then set your own btc hot and cold public key as shown in the figure below.** The **hot public key** is used for general deposit and withdrawal, and the **cold public key** is used to store large amounts of btc to improve security. After becoming a trust and setting up the btc information, the trust will be renewed every 30 days.

![img](https://cdn.jsdelivr.net/gh/hacpy/PictureBed@master/Document/16399849656191639984965606.png)

- proxy_account : An proxy account, if it is not filled in, the default is the same as the council account. Avoid frequent use of council accounts.

- chain: Fill `Bitcoin`

- about: Remark

- hot_entity: Btc public key. Such as `0x029f9830fe29e28064ee2ee57423f000146b75f7f92131d9089e5b395f6e51daf7`.

- cold_entity: Btc public key. Such as `0x033ad05ed2677f49c9591a7c273b5d13afb26c2e964deee403178c053e2149a1fd`.

# Reward distribution

After the renewal of the trust each month, the previous trust can apply to the Treasury for 43750 ksx to the trust multi-signature account. After the ksx is received, any member of the previous trust can distribute rewards through the interface shown in the figure below.

![img](https://cdn.jsdelivr.net/gh/hacpy/PictureBed@master/Document/16399982708971639998270888.png)

- sessionNum: The id of the previous trust.
