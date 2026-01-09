import pytest
from ecdsa import NIST256p, SigningKey, VerifyingKey

from trezorlib import evolu
from trezorlib.debuglink import SessionDebugWrapper as Session
from trezorlib.debuglink import TrezorClientDebugLink as Client
from trezorlib.exceptions import TrezorFailure

from ...common import compact_size
from ..check_signature import check_signature_optiga
from .common import get_delegated_identity_key, sign_proof

pytestmark = pytest.mark.models("core")


def signing_buffer(private_key: bytes, challenge: bytes, size: int) -> bytes:
    public_key: VerifyingKey = SigningKey.from_string(private_key, curve=NIST256p).get_verifying_key()  # type: ignore
    components = [
        b"EvoluSignRegistrationRequestV1:",
        public_key.to_string("uncompressed"),
        challenge,
        size.to_bytes(4, "big"),
    ]
    res = b""
    for comp in components:
        res += compact_size(len(comp)) + comp
    return res


def optiga_unavailable(client: Client) -> bool:
    """Check if Optiga is unavailable by attempting a sign operation."""
    try:
        # empty challenge so the function would raise right after the Optiga check
        challenge = bytes.fromhex("")
        size = 1
        # Dummy proof - the code shouldn't reach proof validation anyway
        proposed_value = bytes.fromhex(
            "1b161be2bfc622b4ffd9943138ab5931e77b4c6835e29b1ac25221c74492495a912c00f488fd5f95b43085f721f36574813785c011c60cf81877ccd05700000000"
        )
        evolu.sign_registration_request(
            client.get_session(),
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )
        return False  # If no exception, Optiga is available
    except TrezorFailure as e:
        if "Optiga is not available" in str(e):
            return True  # Optiga unavailable
        return False  # Different error
    except Exception:
        return False  # Other unexpected errors


@pytest.mark.models("t2t1")
def test_evolu_sign_request_t2t1(session: Session):
    challenge = bytes.fromhex("1234")
    size = 10
    # hardcoded invalid proof but the code shouldn't reach proof validation anyway
    proposed_value = bytes.fromhex(
        "1b161be2bfc622b4ffd9943138ab5931e77b4c6835e29b1ac25221c74492495a912c00f488fd5f95b43085f721f36574813785c011c60cf81877ccd05700000000"
    )

    with pytest.raises(
        TrezorFailure,
        match="Optiga is not available",
    ):
        evolu.sign_registration_request(
            session,
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )


@pytest.mark.models("safe")
def test_evolu_sign_request(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    delegated_identity_key = get_delegated_identity_key(client)
    challenge = bytes.fromhex("1234")
    size = 10
    proposed_value = sign_proof(
        delegated_identity_key,
        b"EvoluSignRegistrationRequest",
        [challenge, size.to_bytes(4, "big")],
    )

    response = evolu.sign_registration_request(
        client.get_session(),
        challenge=challenge,
        size=size,
        proof=proposed_value,
    )

    data = signing_buffer(delegated_identity_key, challenge, size)
    check_signature_optiga(
        response.signature, response.certificate_chain, client.model, data
    )


@pytest.mark.models("safe")
def test_evolu_sign_request_invalid_proof(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    challenge = bytes.fromhex("1234")
    size = 10
    # zeroed last 2 bytes on a hardcoded proof => it should be invalid valid for every test
    proposed_value = bytes.fromhex(
        "20dc125b51c2f596df4a9ae9ef816353dcdbf068b91ac687962742b8bd434276f60258c337e0d03211e599701a87cae8d8ac3258ce01bd484921743c2a5e990000"
    )

    with pytest.raises(
        TrezorFailure,
        match="Invalid proof",
    ):
        evolu.sign_registration_request(
            client.get_session(),
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )


@pytest.mark.models("safe")
def test_evolu_sign_request_challenge_too_long(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    challenge = bytes.fromhex("01" * 300)  # 300 bytes, max is 255
    size = 10
    # hardcoded proof but the code shouldn't reach proof validation anyway
    proposed_value = bytes.fromhex(
        "1fd0b4cd0a04806eaa74ae59cc2f5a740680fc784b877deff6ffa6b9eda7d5a7d4207958c48e679b18c64d0e7fcd0e5be25eb27bcf186fbf9531eb20bc00000000"
    )

    with pytest.raises(
        TrezorFailure,
        match="Invalid challenge length",
    ):
        evolu.sign_registration_request(
            client.get_session(),
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )


@pytest.mark.models("safe")
def test_evolu_sign_request_challenge_too_short(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    challenge = b""  # 0 bytes, minimum is 1
    size = 10
    # hardcoded proof but the code shouldn't reach proof validation anyway
    proposed_value = bytes.fromhex(
        "1fa386d20efb38dbb3f7ae0509651fa36c8128324ef89fa1cfd104e10dced08c594f0e8f0a525a839b4fbfaa92b8c2b51163cef593f5c14fc9f1c8c48d00000000"
    )

    with pytest.raises(
        TrezorFailure,
        match="Invalid challenge length",
    ):
        evolu.sign_registration_request(
            client.get_session(),
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )


@pytest.mark.models("safe")
def test_evolu_sign_request_size_too_small(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    challenge = bytes.fromhex("1234")
    size = -10
    # hardcoded proof but the code shouldn't reach proof validation anyway
    proposed_value = bytes.fromhex(
        "1fa386d20efb38dbb3f7ae0509651fa36c8128324ef89fa1cfd104e10dced08c594f0e8f0a525a839b4fbfaa92b8c2b51163cef593f5c14fc9f1c8c48d00000000"
    )

    with pytest.raises(
        ValueError,
        match=f"Value {size} in field size_to_acquire does not fit into uint32",
    ):
        evolu.sign_registration_request(
            client.get_session(),
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )


@pytest.mark.models("safe")
def test_evolu_sign_request_size_too_large(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    challenge = bytes.fromhex("1234")
    size = 0xFFFFFFFF + 1
    # hardcoded proof but the code shouldn't reach proof validation anyway
    proposed_value = bytes.fromhex(
        "1fa386d20efb38dbb3f7ae0509651fa36c8128324ef89fa1cfd104e10dced08c594f0e8f0a525a839b4fbfaa92b8c2b51163cef593f5c14fc9f1c8c48d00000000"
    )

    with pytest.raises(
        ValueError,
        match=f"Value {size} in field size_to_acquire does not fit into uint32",
    ):
        evolu.sign_registration_request(
            client.get_session(),
            challenge=challenge,
            size=size,
            proof=proposed_value,
        )


@pytest.mark.models("safe")
def test_evolu_sign_request_data_higher_bound(client: Client):
    if optiga_unavailable(client):
        pytest.xfail("Optiga is not available on this device.")
    delegated_identity_key = get_delegated_identity_key(client)
    challenge = bytes.fromhex("12" * 255)
    size = 0xFFFFFFFF
    proposed_value = sign_proof(
        delegated_identity_key,
        b"EvoluSignRegistrationRequest",
        [challenge, size.to_bytes(4, "big")],
    )

    response = evolu.sign_registration_request(
        client.get_session(),
        challenge=challenge,
        size=size,
        proof=proposed_value,
    )

    data = signing_buffer(delegated_identity_key, challenge, size)
    check_signature_optiga(
        response.signature, response.certificate_chain, client.model, data
    )
