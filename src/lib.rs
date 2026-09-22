use solana_program::{
    account_info::{next_account_info, AccountInfo},
    declare_id,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::{clock::Clock, Sysvar},
};

declare_id!("5CP77XBpPsv1K3tsZtx4HsPKxYbavxmMCma6tEcFTdUS");

/// Solscan 认这段魔法字。必须真正编进 ELF，不能靠会被删掉的宏。
#[used]
#[no_mangle]
static SECURITY_TXT: &[u8] = b"=======BEGIN SECURITY.TXT V1=======\0name\0ANT Mine\0project_url\0https://antfm.fun\0contacts\0email:zoeefm@proton.me\0policy\0email zoeefm@proton.me\0source_code\0https://github.com/DEDEvvv/ant-mine\0=======END SECURITY.TXT V1=======\0";

fn keep_security_txt() {
    let _ = core::hint::black_box(&SECURITY_TXT);
}

const MINT: Pubkey = solana_program::pubkey!("D7ahcwSv6GkcBpPJEKizUz8eJFoV4g2FWaUmZHxargan");
const TOKEN_PROGRAM: Pubkey =
    solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const VAULT: &[u8] = b"v";
const USER: &[u8] = b"u";
const MINER: &[u8] = b"m";
const DAY: i64 = 86400;
/// 1 H/s 每天 5555.555555 枚。22 档平均 1.5 H/s，500 台约 12 天挖完 5000 万。
const PER_H_DAY: u64 = 5_555_555_555;
const MAX_MINERS: u16 = 500;
const KINDS: usize = 22;
/// 1.0 H/s ~ 2.0 H/s，最高不超过最低的一倍。
const HASH: [u32; KINDS] = [
    1000, 1048, 1095, 1143, 1190, 1238, 1286, 1333, 1381, 1429, 1476, 1524, 1571, 1619, 1667, 1714,
    1762, 1810, 1857, 1905, 1952, 2000,
];

entrypoint!(process);
fn process(pid: &Pubkey, acc: &[AccountInfo], data: &[u8]) -> ProgramResult {
    keep_security_txt();
    if pid != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    match data.first().copied() {
        Some(0) => init(pid, acc, data),
        Some(1) => mint(pid, acc),
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
    let per_h_day = PER_H_DAY;
    let (cfg_p, bump) = Pubkey::find_program_address(&[VAULT], pid);
    if cfg_a.key != &cfg_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if cfg_a.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let space = 12usize;
    let rent = Rent::get()?.minimum_balance(space);
    invoke_signed(
        &system_instruction::create_account(payer.key, cfg_a.key, rent, space as u64, pid),
        &[payer.clone(), cfg_a.clone(), sys.clone()],
        &[&[VAULT, &[bump]]],
    )?;
    let d = &mut cfg_a.data.borrow_mut();
    d[0] = bump;
    d[1..9].copy_from_slice(&per_h_day.to_le_bytes());
    d[9..11].copy_from_slice(&0u16.to_le_bytes());
    d[11] = 0;
    Ok(())
}

fn roll_kind(owner: &Pubkey, minted: u16, slot: u64) -> u8 {
    let mut seed = slot ^ (minted as u64);
    let bytes = owner.as_ref();
    seed ^= u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    seed ^= u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    seed ^= u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    seed ^= u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    (seed % KINDS as u64) as u8
}

fn mint(pid: &Pubkey, acc: &[AccountInfo]) -> ProgramResult {
    let i = &mut acc.iter();
    let owner = next_account_info(i)?;
    let user_a = next_account_info(i)?;
    let miner_a = next_account_info(i)?;
    let cfg_a = next_account_info(i)?;
    let sys = next_account_info(i)?;
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if cfg_a.data.borrow().len() < 12 {
        return Err(ProgramError::InvalidAccountData);
    }
    let (cfg_p, _) = Pubkey::find_program_address(&[VAULT], pid);
    if cfg_a.key != &cfg_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if cfg_a.data.borrow()[11] != 0 {
        return Err(ProgramError::InvalidArgument);
    }
    let minted = u16::from_le_bytes(cfg_a.data.borrow()[9..11].try_into().unwrap());
    if minted >= MAX_MINERS {
        return Err(ProgramError::InvalidArgument);
    }
    let (u_p, u_b) = Pubkey::find_program_address(&[USER, owner.key.as_ref()], pid);
    if user_a.key != &u_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if user_a.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let (m_p, m_b) = Pubkey::find_program_address(&[MINER, owner.key.as_ref()], pid);
    if miner_a.key != &m_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if miner_a.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let slot = Clock::get()?.slot;
    let kind = roll_kind(owner.key, minted, slot);
    let now = Clock::get()?.unix_timestamp;

    let user_rent = Rent::get()?.minimum_balance(2);
    invoke_signed(
        &system_instruction::create_account(owner.key, user_a.key, user_rent, 2, pid),
        &[owner.clone(), user_a.clone(), sys.clone()],
        &[&[USER, owner.key.as_ref(), &[u_b]]],
    )?;
    {
        let d = &mut user_a.data.borrow_mut();
        d[0] = 1;
        d[1] = kind;
    }

    let miner_rent = Rent::get()?.minimum_balance(42);
    invoke_signed(
        &system_instruction::create_account(owner.key, miner_a.key, miner_rent, 42, pid),
        &[owner.clone(), miner_a.clone(), sys.clone()],
        &[&[MINER, owner.key.as_ref(), &[m_b]]],
    )?;
    {
        let d = &mut miner_a.data.borrow_mut();
        d[0..32].copy_from_slice(owner.key.as_ref());
        d[32] = kind;
        d[33..41].copy_from_slice(&now.to_le_bytes());
        d[41] = 0;
    }

    let next = minted.saturating_add(1);
    cfg_a.data.borrow_mut()[9..11].copy_from_slice(&next.to_le_bytes());
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
    if cfg_a.data.borrow().len() < 12 {
        return Err(ProgramError::InvalidAccountData);
    }
    let ended = cfg_a.data.borrow()[11] != 0;
    let bump = cfg_a.data.borrow()[0];
    let per_h_day = u64::from_le_bytes(cfg_a.data.borrow()[1..9].try_into().unwrap());
    let (cfg_p, _) = Pubkey::find_program_address(&[VAULT], pid);
    if cfg_a.key != &cfg_p {
        return Err(ProgramError::InvalidSeeds);
    }
    let (m_p, _) = Pubkey::find_program_address(&[MINER, owner.key.as_ref()], pid);
    if miner_a.key != &m_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if miner_a.data.borrow().len() < 42 {
        return Err(ProgramError::InvalidAccountData);
    }
    let miner_dead = miner_a.data.borrow()[41] != 0;
    if ended || miner_dead {
        miner_a.data.borrow_mut()[41] = 1;
        return Ok(());
    }
    let miner_owner = Pubkey::new_from_array(miner_a.data.borrow()[0..32].try_into().unwrap());
    if miner_owner != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }
    let kind = miner_a.data.borrow()[32];
    let last = i64::from_le_bytes(miner_a.data.borrow()[33..41].try_into().unwrap());
    if (kind as usize) >= HASH.len() {
        return Err(ProgramError::InvalidAccountData);
    }
    let now = Clock::get()?.unix_timestamp;
    if now <= last {
        return Ok(());
    }
    let hash = HASH[kind as usize];
    let elapsed = (now - last) as u128;
    let amt = (hash as u128)
        .saturating_mul(elapsed)
        .saturating_mul(per_h_day as u128)
        / 1000u128
        / (DAY as u128);
    if amt == 0 {
        return Ok(());
    }
    let vd = vault_ata.data.borrow();
    if vd.len() < 72 {
        return Err(ProgramError::InvalidAccountData);
    }
    let mint = Pubkey::new_from_array(vd[0..32].try_into().unwrap());
    if mint != MINT {
        return Err(ProgramError::InvalidAccountData);
    }
    let vault_amt = u64::from_le_bytes(vd[64..72].try_into().unwrap());
    drop(vd);
    let pay = core::cmp::min(amt, vault_amt as u128) as u64;
    if pay == 0 {
        return Ok(());
    }
    if token_p.key != &TOKEN_PROGRAM {
        return Err(ProgramError::IncorrectProgramId);
    }
    let dd = dest.data.borrow();
    if dd.len() < 72 {
        return Err(ProgramError::InvalidAccountData);
    }
    let dest_mint = Pubkey::new_from_array(dd[0..32].try_into().unwrap());
    if dest_mint != MINT {
        return Err(ProgramError::InvalidAccountData);
    }
    let dest_owner = Pubkey::new_from_array(dd[32..64].try_into().unwrap());
    if dest_owner != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }
    drop(dd);
    miner_a.data.borrow_mut()[33..41].copy_from_slice(&now.to_le_bytes());
    let mut ix_data = [0u8; 9];
    ix_data[0] = 3;
    ix_data[1..9].copy_from_slice(&pay.to_le_bytes());
    let ix = Instruction {
        program_id: TOKEN_PROGRAM,
        accounts: vec![
            AccountMeta::new(*vault_ata.key, false),
            AccountMeta::new(*dest.key, false),
            AccountMeta::new_readonly(cfg_p, true),
        ],
        data: ix_data.to_vec(),
    };
    invoke_signed(
        &ix,
        &[vault_ata.clone(), dest.clone(), cfg_a.clone(), token_p.clone()],
        &[&[VAULT, &[bump]]],
    )?;
    if pay == vault_amt {
        cfg_a.data.borrow_mut()[11] = 1;
        miner_a.data.borrow_mut()[41] = 1;
    }
    Ok(())
}
