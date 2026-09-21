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

declare_id!("APNv3yTr5Zd5UYrv2LpHnfoLQ1WKA8K3gZ1Sk3DZeZL5");

/// Solscan 认这段魔法字。必须真正编进 ELF，不能靠会被删掉的宏。
#[used]
#[no_mangle]
static SECURITY_TXT: &[u8] = b"=======BEGIN SECURITY.TXT V1=======\0name\0ANT Mine\0project_url\0https://antfm.fun\0contacts\0email:zoeefm@proton.me\0policy\0email zoeefm@proton.me\0source_code\0https://github.com/DEDEvvv/ant-mine\0=======END SECURITY.TXT V1=======\0";

fn keep_security_txt() {
    let _ = core::hint::black_box(&SECURITY_TXT);
}

const MINT: Pubkey = solana_program::pubkey!("7PrUyJot9dKnuycunNwBLQQS84fiqTPE7XcfWdtYcgan");
const TREASURY: Pubkey = solana_program::pubkey!("w9n9KrpSjzUKyQi4bUtyKn8FfTops5kUQc7xFYp4iS6");
const TOKEN_PROGRAM: Pubkey =
    solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const VAULT: &[u8] = b"v";
const USER: &[u8] = b"u";
const MINER: &[u8] = b"m";
const STARTER: u8 = 1;
const DAY: i64 = 86400;
const HASH: [u32; 22] = [
    500, 1000, 1600, 2500, 3800, 5500, 4800, 8000, 12000, 17000, 24000, 34000, 48000, 66000, 90000,
    125000, 170000, 230000, 310000, 410000, 530000, 680000,
];
const PRICE: [u64; 22] = [
    200_000_000,
    0,
    1_200_000_000,
    1_700_000_000,
    2_400_000_000,
    3_200_000_000,
    2_800_000_000,
    4_200_000_000,
    6_000_000_000,
    8_000_000_000,
    10_000_000_000,
    13_000_000_000,
    17_000_000_000,
    22_000_000_000,
    28_000_000_000,
    36_000_000_000,
    46_000_000_000,
    58_000_000_000,
    72_000_000_000,
    88_000_000_000,
    104_000_000_000,
    120_000_000_000,
];

entrypoint!(process);
fn process(pid: &Pubkey, acc: &[AccountInfo], data: &[u8]) -> ProgramResult {
    keep_security_txt();
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
    let space = 9usize;
    let rent = Rent::get()?.minimum_balance(space);
    invoke_signed(
        &system_instruction::create_account(payer.key, cfg_a.key, rent, space as u64, pid),
        &[payer.clone(), cfg_a.clone(), sys.clone()],
        &[&[VAULT, &[bump]]],
    )?;
    let d = &mut cfg_a.data.borrow_mut();
    d[0] = bump;
    d[1..9].copy_from_slice(&per_h_day.to_le_bytes());
    Ok(())
}

fn buy(pid: &Pubkey, acc: &[AccountInfo], data: &[u8]) -> ProgramResult {
    if data.len() < 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let kind = data[1];
    if (kind as usize) >= HASH.len() {
        return Err(ProgramError::InvalidArgument);
    }
    let price = PRICE[kind as usize];
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
        let rent = Rent::get()?.minimum_balance(5);
        invoke_signed(
            &system_instruction::create_account(owner.key, user_a.key, rent, 5, pid),
            &[owner.clone(), user_a.clone(), sys.clone()],
            &[&[USER, owner.key.as_ref(), &[u_b]]],
        )?;
        let d = &mut user_a.data.borrow_mut();
        d[..5].fill(0);
    }
    let mut n = u32::from_le_bytes(user_a.data.borrow()[0..4].try_into().unwrap());
    let mut starter = user_a.data.borrow()[4];
    if kind == STARTER {
        if starter != 0 {
            return Err(ProgramError::InvalidArgument);
        }
        starter = 1;
    }
    let id = n;
    n = id.saturating_add(1);
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
    let space = 41usize;
    let rent = Rent::get()?.minimum_balance(space);
    invoke_signed(
        &system_instruction::create_account(owner.key, miner_a.key, rent, space as u64, pid),
        &[owner.clone(), miner_a.clone(), sys.clone()],
        &[&[MINER, owner.key.as_ref(), &id.to_le_bytes(), &[m_b]]],
    )?;
    {
        let d = &mut miner_a.data.borrow_mut();
        d[0..32].copy_from_slice(owner.key.as_ref());
        d[32] = kind;
        d[33..41].copy_from_slice(&now.to_le_bytes());
    }
    {
        let d = &mut user_a.data.borrow_mut();
        d[0..4].copy_from_slice(&n.to_le_bytes());
        d[4] = starter;
    }
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
    if cfg_a.data.borrow().len() < 9 {
        return Err(ProgramError::InvalidAccountData);
    }
    let bump = cfg_a.data.borrow()[0];
    let per_h_day = u64::from_le_bytes(cfg_a.data.borrow()[1..9].try_into().unwrap());
    let (cfg_p, _) = Pubkey::find_program_address(&[VAULT], pid);
    if cfg_a.key != &cfg_p {
        return Err(ProgramError::InvalidSeeds);
    }
    if miner_a.data.borrow().len() < 41 {
        return Err(ProgramError::InvalidAccountData);
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
    miner_a.data.borrow_mut()[33..41].copy_from_slice(&now.to_le_bytes());
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
    Ok(())
}
