use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    declare_id,
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::instruction as token_ix;
use spl_token::state::Account as TokenAccount;
use solana_security_txt::security_txt;

declare_id!("APNv3yTr5Zd5UYrv2LpHnfoLQ1WKA8K3gZ1Sk3DZeZL5");

security_txt! {
    name: "ANT Mine",
    project_url: "https://antfm.fun",
    contacts: "email:zoeefm@proton.me",
    policy: "Report vulnerabilities by email to zoeefm@proton.me. Do not open public issues for unfixed vulnerabilities.",
    preferred_languages: "zh,en"
}

const MINT: Pubkey = solana_program::pubkey!("7PrUyJot9dKnuycunNwBLQQS84fiqTPE7XcfWdtYcgan");
const TREASURY: Pubkey = solana_program::pubkey!("w9n9KrpSjzUKyQi4bUtyKn8FfTops5kUQc7xFYp4iS6");
const VAULT: &[u8] = b"v";
const USER: &[u8] = b"u";
const MINER: &[u8] = b"m";
const STARTER: u8 = 1;
const DAY: i64 = 86400;
/// hash_milli, lamports. 与现图鉴一致；0.5H=500。
const CAT: [(u32, u64); 22] = [
    (500, 200_000_000),
    (1000, 0),
    (1600, 1_200_000_000),
    (2500, 1_700_000_000),
    (3800, 2_400_000_000),
    (5500, 3_200_000_000),
    (4800, 2_800_000_000),
    (8000, 4_200_000_000),
    (12000, 6_000_000_000),
    (17000, 8_000_000_000),
    (24000, 10_000_000_000),
    (34000, 13_000_000_000),
    (48000, 17_000_000_000),
    (66000, 22_000_000_000),
    (90000, 28_000_000_000),
    (125000, 36_000_000_000),
    (170000, 46_000_000_000),
    (230000, 58_000_000_000),
    (310000, 72_000_000_000),
    (410000, 88_000_000_000),
    (530000, 104_000_000_000),
    (680000, 120_000_000_000),
];

#[derive(BorshSerialize, BorshDeserialize)]
struct Cfg {
    bump: u8,
    /// 每 1000 hash_milli（=1H）每天产出的最小单位。初始化写死，之后不能改。
    per_h_day: u64,
}
#[derive(BorshSerialize, BorshDeserialize)]
struct User {
    n: u32,
    starter: u8,
}
#[derive(BorshSerialize, BorshDeserialize)]
struct Miner {
    owner: Pubkey,
    kind: u8,
    last: i64,
}

entrypoint!(process);
fn process(pid: &Pubkey, acc: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if pid != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    match data.first().copied() {
        Some(0) => init(pid, acc, data),
        Some(1) => buy(pid, acc, data),
        Some(2) => claim(pid, acc),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn init(pid: &Pubkey, acc: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let i = &mut acc.iter();
    let payer = next_account_info(i)?;
    let cfg_a = next_account_info(i)?;
    let sys = next_account_info(i)?;
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if data.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let per_h_day = u64::from_le_bytes(data[1..9].try_into().unwrap());
    if per_h_day == 0 {
        return Err(ProgramError::InvalidArgument);
    }
    let (cfg_p, bump) = Pubkey::find_program_address(&[VAULT], pid);
    if cfg_a.key != &cfg_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if cfg_a.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let space = 1 + 8;
    let rent = Rent::get()?.minimum_balance(space);
    invoke_signed(
        &system_instruction::create_account(payer.key, cfg_a.key, rent, space as u64, pid),
        &[payer.clone(), cfg_a.clone(), sys.clone()],
        &[&[VAULT, &[bump]]],
    )?;
    Cfg { bump, per_h_day }.serialize(&mut &mut cfg_a.data.borrow_mut()[..])?;
    Ok(())
}

fn buy(pid: &Pubkey, acc: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if data.len() < 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let kind = data[1];
    if (kind as usize) >= CAT.len() {
        return Err(ProgramError::InvalidArgument);
    }
    let (hash, price) = CAT[kind as usize];
    let _ = hash;
    let i = &mut acc.iter();
    let owner = next_account_info(i)?;
    let user_a = next_account_info(i)?;
    let miner_a = next_account_info(i)?;
    let treasury = next_account_info(i)?;
    let sys = next_account_info(i)?;
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if treasury.key != &TREASURY {
        return Err(ProgramError::InvalidAccountData);
    }
    let (u_p, u_b) = Pubkey::find_program_address(&[USER, owner.key.as_ref()], pid);
    if user_a.key != &u_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if user_a.lamports() == 0 {
        let rent = Rent::get()?.minimum_balance(4 + 1);
        invoke_signed(
            &system_instruction::create_account(owner.key, user_a.key, rent, 5, pid),
            &[owner.clone(), user_a.clone(), sys.clone()],
            &[&[USER, owner.key.as_ref(), &[u_b]]],
        )?;
        User { n: 0, starter: 0 }.serialize(&mut &mut user_a.data.borrow_mut()[..])?;
    }
    let mut u = User::try_from_slice(&user_a.data.borrow())?;
    if kind == STARTER {
        if u.starter != 0 {
            return Err(ProgramError::InvalidArgument);
        }
        u.starter = 1;
    }
    let id = u.n;
    u.n = id.saturating_add(1);
    if price > 0 {
        invoke(
            &system_instruction::transfer(owner.key, treasury.key, price),
            &[owner.clone(), treasury.clone(), sys.clone()],
        )?;
    }
    let (m_p, m_b) = Pubkey::find_program_address(&[MINER, owner.key.as_ref(), &id.to_le_bytes()], pid);
    if miner_a.key != &m_p {
        return Err(ProgramError::InvalidSeeds);
    }
    let now = Clock::get()?.unix_timestamp;
    let space = 32 + 1 + 8;
    let rent = Rent::get()?.minimum_balance(space);
    invoke_signed(
        &system_instruction::create_account(owner.key, miner_a.key, rent, space as u64, pid),
        &[owner.clone(), miner_a.clone(), sys.clone()],
        &[&[MINER, owner.key.as_ref(), &id.to_le_bytes(), &[m_b]]],
    )?;
    Miner { owner: *owner.key, kind, last: now }.serialize(&mut &mut miner_a.data.borrow_mut()[..])?;
    u.serialize(&mut &mut user_a.data.borrow_mut()[..])?;
    Ok(())
}

fn claim(pid: &Pubkey, acc: &[AccountInfo]) -> ProgramResult {
    let i = &mut acc.iter();
    let owner = next_account_info(i)?;
    let miner_a = next_account_info(i)?;
    let cfg_a = next_account_info(i)?;
    let vault_ata = next_account_info(i)?;
    let dest = next_account_info(i)?;
    let token_p = next_account_info(i)?;
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let cfg = Cfg::try_from_slice(&cfg_a.data.borrow())?;
    let (cfg_p, _) = Pubkey::find_program_address(&[VAULT], pid);
    if cfg_a.key != &cfg_p {
        return Err(ProgramError::InvalidSeeds);
    }
    let mut m = Miner::try_from_slice(&miner_a.data.borrow())?;
    if m.owner != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }
    let now = Clock::get()?.unix_timestamp;
    if now <= m.last {
        return Ok(());
    }
    let (hash, _) = CAT[m.kind as usize];
    let elapsed = (now - m.last) as u128;
    let amt = (hash as u128)
        .saturating_mul(elapsed)
        .saturating_mul(cfg.per_h_day as u128)
        / 1000u128
        / (DAY as u128);
    m.last = now;
    m.serialize(&mut &mut miner_a.data.borrow_mut()[..])?;
    if amt == 0 {
        return Ok(());
    }
    let vault = TokenAccount::unpack(&vault_ata.data.borrow())?;
    if vault.mint != MINT {
        return Err(ProgramError::InvalidAccountData);
    }
    let pay = core::cmp::min(amt, vault.amount as u128) as u64;
    if pay == 0 {
        return Ok(());
    }
    invoke_signed(
        &token_ix::transfer(
            token_p.key,
            vault_ata.key,
            dest.key,
            &cfg_p,
            &[],
            pay,
        )?,
        &[vault_ata.clone(), dest.clone(), cfg_a.clone(), token_p.clone()],
        &[&[VAULT, &[cfg.bump]]],
    )?;
    Ok(())
}
