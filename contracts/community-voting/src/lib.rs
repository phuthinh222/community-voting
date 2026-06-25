#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short,
    Address, Env, String,
};

const DAY: u32 = 17280;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    ProposalCount,
    Proposal(u64),
    HasVoted(u64, Address),
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub yes_count: u64,
    pub no_count: u64,
    pub active: bool,
    pub created_at: u64,
    pub deadline: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    AlreadyVoted = 2,
    ProposalNotFound = 3,
    ProposalClosed = 4,
    NotAuthorized = 5,
    VotingEnded = 6,
}

#[contract]
pub struct CommunityVoting;

#[contractimpl]
impl CommunityVoting {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::ProposalCount, &0_u64);
        env.storage().instance().extend_ttl(6 * DAY, 7 * DAY);

        Ok(())
    }

    pub fn create_proposal(
        env: Env,
        caller: Address,
        title: String,
        description: String,
        deadline: u64,
    ) -> Result<u64, Error> {
        caller.require_auth();

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);

        let id = count + 1;

        let proposal = Proposal {
            id,
            title,
            description,
            yes_count: 0,
            no_count: 0,
            active: true,
            created_at: env.ledger().timestamp(),
            deadline,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(id), &proposal);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Proposal(id), 29 * DAY, 30 * DAY);

        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &id);

        env.storage().instance().extend_ttl(6 * DAY, 7 * DAY);

        env.events().publish((symbol_short!("create"),), id);

        Ok(id)
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    pub fn vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        vote_yes: bool,
    ) -> Result<(), Error> {
        voter.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::HasVoted(proposal_id, voter.clone()))
        {
            return Err(Error::AlreadyVoted);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        if !proposal.active {
            return Err(Error::ProposalClosed);
        }

        if env.ledger().timestamp() > proposal.deadline {
            return Err(Error::VotingEnded);
        }

        if vote_yes {
            proposal.yes_count += 1;
        } else {
            proposal.no_count += 1;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Proposal(proposal_id), 29 * DAY, 30 * DAY);

        env.storage()
            .persistent()
            .set(&DataKey::HasVoted(proposal_id, voter.clone()), &true);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::HasVoted(proposal_id, voter), 29 * DAY, 30 * DAY);

        env.events()
            .publish((symbol_short!("vote"),), (proposal_id, vote_yes));

        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::HasVoted(proposal_id, voter))
    }

    pub fn result(env: Env, proposal_id: u64) -> Result<String, Error> {
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        let outcome = if proposal.yes_count > proposal.no_count {
            "PASSED"
        } else if proposal.no_count > proposal.yes_count {
            "REJECTED"
        } else {
            "TIED"
        };

        Ok(String::from_str(&env, outcome))
    }

    pub fn close_proposal(env: Env, caller: Address, proposal_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        if caller != admin {
            return Err(Error::NotAuthorized);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        proposal.active = false;

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Proposal(proposal_id), 29 * DAY, 30 * DAY);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

    fn setup() -> (Env, CommunityVotingClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|l| l.timestamp = 1_000_000);

        let contract_id = env.register(CommunityVoting, ());
        let client = CommunityVotingClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        (env, client, admin)
    }

    #[test]
    fn test_voting_flow() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Build a new gym"),
            &String::from_str(&env, "Budget 1000 XLM for construction"),
            &9_999_999_u64,
        );

        assert_eq!(id, 1);
        assert_eq!(client.proposal_count(), 1);

        client.vote(&alice, &id, &true);
        client.vote(&bob, &id, &false);

        let proposal = client.get_proposal(&id);
        assert_eq!(proposal.yes_count, 1);
        assert_eq!(proposal.no_count, 1);
        assert!(proposal.active);
        assert_eq!(proposal.created_at, 1_000_000);
        assert_eq!(
            proposal.description,
            String::from_str(&env, "Budget 1000 XLM for construction")
        );
    }

    #[test]
    #[should_panic]
    fn test_duplicate_vote() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Install solar panels"),
            &String::from_str(&env, "Reduce energy cost by 50%"),
            &9_999_999_u64,
        );

        client.vote(&alice, &id, &true);

        // Verify first vote counted before expecting the panic
        let p = client.get_proposal(&id);
        assert_eq!(p.yes_count, 1);

        // Second vote by same voter must panic (AlreadyVoted)
        client.vote(&alice, &id, &true);
    }

    #[test]
    #[should_panic]
    fn test_closed_proposal_vote() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Plant more trees"),
            &String::from_str(&env, "500 trees around the park"),
            &9_999_999_u64,
        );

        client.close_proposal(&admin, &id);

        let p = client.get_proposal(&id);
        assert!(!p.active);

        // Voting on a closed proposal must panic (ProposalClosed)
        client.vote(&alice, &id, &true);
    }

    #[test]
    fn test_multiple_proposals() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id1 = client.create_proposal(
            &admin,
            &String::from_str(&env, "Proposal One"),
            &String::from_str(&env, "First proposal description"),
            &9_999_999_u64,
        );
        let id2 = client.create_proposal(
            &admin,
            &String::from_str(&env, "Proposal Two"),
            &String::from_str(&env, "Second proposal description"),
            &9_999_999_u64,
        );

        assert_eq!(client.proposal_count(), 2);

        client.vote(&alice, &id1, &true);
        client.vote(&bob, &id2, &false);

        let p1 = client.get_proposal(&id1);
        let p2 = client.get_proposal(&id2);

        assert_eq!(p1.yes_count, 1);
        assert_eq!(p2.no_count, 1);
    }

    #[test]
    #[should_panic]
    fn test_close_not_authorized() {
        let (env, client, admin) = setup();

        let _ = admin;
        let stranger = Address::generate(&env);
        let id = client.create_proposal(
            &stranger,
            &String::from_str(&env, "Hack the vote"),
            &String::from_str(&env, "Bad proposal"),
            &9_999_999_u64,
        );

        // Only admin can close — stranger must panic (NotAuthorized)
        client.close_proposal(&stranger, &id);
    }

    #[test]
    #[should_panic]
    fn test_voting_ended() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        // deadline = 500_000 < ledger timestamp 1_000_000 → already expired
        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Expired proposal"),
            &String::from_str(&env, "This vote has already closed"),
            &500_000_u64,
        );

        // Voting after deadline must panic (VotingEnded)
        client.vote(&alice, &id, &true);
    }

    #[test]
    fn test_result() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let carol = Address::generate(&env);

        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Fund new library"),
            &String::from_str(&env, "Build a community library"),
            &9_999_999_u64,
        );

        assert_eq!(client.result(&id), String::from_str(&env, "TIED"));

        client.vote(&alice, &id, &true);
        client.vote(&bob, &id, &true);
        client.vote(&carol, &id, &false);

        assert_eq!(client.result(&id), String::from_str(&env, "PASSED"));
    }

    #[test]
    fn test_result_rejected() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Ban social media"),
            &String::from_str(&env, "Restrict social media access"),
            &9_999_999_u64,
        );

        client.vote(&alice, &id, &false);
        client.vote(&bob, &id, &false);

        assert_eq!(client.result(&id), String::from_str(&env, "REJECTED"));
    }

    #[test]
    fn test_has_voted() {
        let (env, client, admin) = setup();

        let alice = Address::generate(&env);

        let id = client.create_proposal(
            &admin,
            &String::from_str(&env, "Any proposal"),
            &String::from_str(&env, "Some description"),
            &9_999_999_u64,
        );

        assert!(!client.has_voted(&id, &alice));

        client.vote(&alice, &id, &true);

        assert!(client.has_voted(&id, &alice));
    }
}
