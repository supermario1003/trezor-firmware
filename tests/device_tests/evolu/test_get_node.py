import pytest

from trezorlib import evolu
from trezorlib.debuglink import SessionDebugWrapper as Session
from trezorlib.debuglink import TrezorClientDebugLink as Client
from trezorlib.exceptions import TrezorFailure

from .common import get_delegated_identity_key, sign_proof

pytestmark = [
    pytest.mark.models("core"),
    # the tests vectors in this test are for the SLIP-14 seed. It should be initialized from `conftest.py` already but we set it explicitly to be sure
    pytest.mark.setup_client(
        mnemonic="all all all all all all all all all all all all", passphrase=False
    ),
]


def test_evolu_get_node(client: Client):
    delegated_identity_key = get_delegated_identity_key(client)
    proof = sign_proof(delegated_identity_key, b"EvoluGetNode", [])
    node = evolu.get_node(client.get_session(), proof=proof)

    # expected node for the SLIP-14 seed
    check_value = bytes.fromhex(
        "a81aaf51997b6ddfa33d11c038d6aba5f711754a2c823823ff8b777825cdbb32b0e71c301fa381c75081bd3bcc134b63306aa6fc9a9f52d835ad4df8cd507be6"
    )
    assert node == check_value


@pytest.mark.setup_client(
    # a different seed
    mnemonic="valve multiply shuffle venue then cruel genre venture fruit hammer sponsor luxury",
    passphrase=False,
)
def test_evolu_get_node_different_seed(client: Client):
    delegated_identity_key = get_delegated_identity_key(client)
    proof = sign_proof(delegated_identity_key, b"EvoluGetNode", [])
    node = evolu.get_node(client.get_session(), proof=proof)

    # expected node for the SLIP-14 seed
    check_value = bytes.fromhex(
        "a81aaf51997b6ddfa33d11c038d6aba5f711754a2c823823ff8b777825cdbb32b0e71c301fa381c75081bd3bcc134b63306aa6fc9a9f52d835ad4df8cd507be6"
    )

    # check that the generated node is different
    assert node != check_value


def test_evolu_get_node_invalid_proof(client: Client):
    delegated_identity_key = get_delegated_identity_key(client)
    proof = sign_proof(delegated_identity_key, b"EvoluGetNode", [])
    # tamper with the proof to make it invalid
    invalid_proof = proof[:-2] + bytes([proof[-2] ^ 0xFFFF])

    with pytest.raises(
        TrezorFailure,
        match="Invalid proof",
    ):
        evolu.get_node(client.get_session(), proof=invalid_proof)


def test_evolu_get_node_no_proof(client: Client):
    with pytest.raises(
        TrezorFailure,
        match="Invalid proof",
    ):
        evolu.get_node(client.get_session(), proof=b"")


def test_evolu_get_node_none_proof(client: Client):
    with pytest.raises(
        TrezorFailure,
        match="DataError: Failed to decode message: Missing required field. proof_of_delegated_identity",
    ):
        evolu.get_node(client.get_session(), proof=None)  # type: ignore
