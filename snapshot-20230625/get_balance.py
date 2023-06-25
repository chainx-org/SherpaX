from web3 import Web3
import json

# 连接到sherpax节点
w3 = Web3(Web3.HTTPProvider('http://192.168.31.227:8546'))
SNAPSHOT_BLOCKNUMBER = 7559500
EXISTENTIAL_DEPOSIT = 10000000000


def is_contract_address(address):
    return b'' != w3.eth.getCode(address)


def filter_accounts():
    accounts = []

    with open("sherpax_all_evm_addresses.csv", 'r') as file:
        for (i, line) in enumerate(file):
            address = "0x" + line.strip().replace('"', '')  # 去除行尾的换行符和空格

            if address == "0x0000000000000000000000000000000000000000":
                continue

            if not is_contract_address(address):
                accounts.append(address)
                print(i)

    save_data("accounts-15786.json", sorted(set(accounts)))


def get_accounts():
    with open('./accounts-15786.json', 'r') as file:
        data = json.load(file)

        print(len(data))

        return data


def get_ksx_balance(account):
    return w3.eth.get_balance(account, SNAPSHOT_BLOCKNUMBER)


def get_so_balance(account):
    so_contract = "0xF373b95a00662ed1211948F414b252E56c0fa0C4"

    # ERC-20 代币余额查询的 ABI
    abi = [
        {
            "constant": True,
            "inputs": [{"name": "_owner", "type": "address"}],
            "name": "balanceOf",
            "outputs": [{"name": "balance", "type": "uint256"}],
            "payable": False,
            "stateMutability": "view",
            "type": "function"
        }
    ]

    # 创建合约实例
    contract = w3.eth.contract(address=so_contract, abi=abi)

    # 调用合约方法查询余额
    return contract.functions.balanceOf(account).call(block_identifier=SNAPSHOT_BLOCKNUMBER)


def save_data(filename, data):
    with open(filename, "w") as file:
        json.dump(data, file, indent=2, sort_keys=True)


def snapshot_ksx():
    records = []
    total_balance = 0

    for (i, account) in enumerate(get_accounts()):
        record = {}

        ksx_balance = get_ksx_balance(account)

        if ksx_balance < 1000000000000000000:
            continue

        record["address"] = account
        record["ksx"] = ksx_balance + EXISTENTIAL_DEPOSIT

        total_balance += ksx_balance

        records.append(record)
        print(i, account, record["ksx"])

    file_name = "snapshot-ksx-{}-{}".format(len(records), total_balance)
    save_data(file_name, records)


def snapshot_so():
    records = []
    total_balance = 0

    for (i, account) in enumerate(get_accounts()):
        record = {}

        so_balance = get_so_balance(account)

        if so_balance == 0:
            continue

        record["address"] = account
        record["so"] = so_balance

        records.append(record)

        total_balance += so_balance

        print(i, account, w3.fromWei(record["so"], 'ether'))

    file_name = "snapshot-so-{}-{}".format(len(records), total_balance)
    save_data(file_name, records)


if __name__ == "__main__":
    # snapshot_ksx()
    snapshot_so()
