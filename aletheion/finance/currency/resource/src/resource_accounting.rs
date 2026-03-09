#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const RESOURCE_ACCOUNTING_VERSION: u32 = 20260310;
pub const MAX_ACCOUNTS: usize = 262144;
pub const MAX_TRANSACTIONS: usize = 1048576;
pub const MAX_RESOURCE_TYPES: usize = 256;
pub const PHOENIX_BASIC_INCOME_USD_MONTH: f64 = 1500.0;
pub const RESOURCE_BACKING_RATIO: f64 = 0.75;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurrencyType {
    AletheionCredit = 0, TimeBank = 1, ResourceNote = 2, EnergyToken = 3,
    WaterCredit = 4, FoodToken = 5, HousingCredit = 6, EducationToken = 7,
    HealthcareCredit = 8, TransitToken = 9, CarbonCredit = 10, WasteCredit = 11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransactionType {
    Transfer = 0, Exchange = 1, Issuance = 2, Redemption = 3,
    Tax = 4, Dividend = 5, Penalty = 6, Refund = 7,
    Donation = 8, Loan = 9, Repayment = 10, Stake = 11,
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceAccount {
    pub account_id: u64,
    pub owner_did: [u8; 32],
    pub account_type: u8,
    pub balances: [f64; 12],
    pub resource_holdings: [f64; 16],
    pub credit_score: f64,
    pub reputation_score: f64,
    pub basic_income_eligible: bool,
    pub last_transaction_ns: u64,
    pub created_at_ns: u64,
    pub frozen: bool,
    pub audit_hash: [u8; 32],
}

impl ResourceAccount {
    pub fn get_balance(&self, currency: CurrencyType) -> f64 {
        self.balances[currency as usize]
    }
    pub fn get_resource_holding(&self, resource_idx: usize) -> f64 {
        if resource_idx < 16 { self.resource_holdings[resource_idx] } else { 0.0 }
    }
    pub fn can_transact(&self, now_ns: u64) -> bool {
        !self.frozen && now_ns - self.last_transaction_ns < 31536000000000000
    }
    pub fn compute_total_wealth(&self) -> f64 {
        let mut total = 0.0;
        for i in 0..12 { total += self.balances[i]; }
        for i in 0..16 { total += self.resource_holdings[i] * 10.0; }
        total
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transaction {
    pub tx_id: u64,
    pub tx_type: TransactionType,
    pub from_account: u64,
    pub to_account: u64,
    pub currency: CurrencyType,
    pub amount: f64,
    pub resource_backing: f64,
    pub timestamp_ns: u64,
    pub block_height: u64,
    pub signature: [u8; 64],
    pub verified: bool,
    pub finalized: bool,
    pub reversal_prohibited: bool,
}

impl Transaction {
    pub fn is_valid(&self, now_ns: u64) -> bool {
        self.verified && self.finalized && !self.reversal_prohibited &&
        now_ns - self.timestamp_ns < 31536000000000000
    }
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hash = [0u8; 32];
        for i in 0..8 { hash[i] = ((self.tx_id >> (i * 8)) & 0xFF) as u8; }
        hash[8] = self.tx_type as u8;
        hash[9] = self.currency as u8;
        for i in 0..8 { hash[10 + i] = ((self.amount as u64 >> (i * 8)) & 0xFF) as u8; }
        hash
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceBacking {
    pub resource_type: u8,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub reserved_supply: f64,
    pub backing_ratio: f64,
    pub last_audit_ns: u64,
    pub audit_verified: bool,
}

pub struct CommunityCurrencySystem {
    pub system_id: u64,
    pub city_code: [u8; 8],
    pub accounts: [Option<ResourceAccount>; MAX_ACCOUNTS],
    pub account_count: usize,
    pub transactions: [Option<Transaction>; MAX_TRANSACTIONS],
    pub tx_count: usize,
    pub resource_backings: [Option<ResourceBacking>; MAX_RESOURCE_TYPES],
    pub backing_count: usize,
    pub total_currency_in_circulation: [f64; 12],
    pub total_resource_value: f64,
    pub basic_income_distributed_usd: f64,
    pub basic_income_recipients: u64,
    pub gini_coefficient: f64,
    pub economic_velocity: f64,
    pub inflation_rate_pct: f64,
    pub last_distribution_ns: u64,
    pub audit_checksum: u64,
}

impl CommunityCurrencySystem {
    pub fn new(system_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            system_id,
            city_code,
            accounts: Default::default(),
            account_count: 0,
            transactions: Default::default(),
            tx_count: 0,
            resource_backings: Default::default(),
            backing_count: 0,
            total_currency_in_circulation: [0.0; 12],
            total_resource_value: 0.0,
            basic_income_distributed_usd: 0.0,
            basic_income_recipients: 0,
            gini_coefficient: 0.0,
            economic_velocity: 0.0,
            inflation_rate_pct: 0.0,
            last_distribution_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn create_account(&mut self, owner_did: [u8; 32], account_type: u8, now_ns: u64) -> Result<u64, &'static str> {
        if self.account_count >= MAX_ACCOUNTS { return Err("ACCOUNT_LIMIT_EXCEEDED"); }
        let account = ResourceAccount {
            account_id: self.account_count as u64,
            owner_did,
            account_type,
            balances: [0.0; 12],
            resource_holdings: [0.0; 16],
            credit_score: 500.0,
            reputation_score: 100.0,
            basic_income_eligible: account_type == 0,
            last_transaction_ns: now_ns,
            created_at_ns: now_ns,
            frozen: false,
            audit_hash: [0u8; 32],
        };
        self.accounts[self.account_count] = Some(account);
        self.account_count += 1;
        self.update_audit_checksum();
        Ok(account.account_id)
    }
    pub fn process_transaction(&mut self, mut tx: Transaction, now_ns: u64) -> Result<u64, &'static str> {
        if self.tx_count >= MAX_TRANSACTIONS { return Err("TRANSACTION_LIMIT_EXCEEDED"); }
        let from_account = self.accounts.iter()
            .filter_map(|a| a.as_ref())
            .find(|a| a.account_id == tx.from_account)
            .ok_or("FROM_ACCOUNT_NOT_FOUND")?;
        let to_account = self.accounts.iter()
            .filter_map(|a| a.as_ref())
            .find(|a| a.account_id == tx.to_account)
            .ok_or("TO_ACCOUNT_NOT_FOUND")?;
        if !from_account.can_transact(now_ns) { return Err("FROM_ACCOUNT_FROZEN"); }
        if from_account.get_balance(tx.currency) < tx.amount { return Err("INSUFFICIENT_BALANCE"); }
        if tx.resource_backing > 0.0 && tx.resource_backing < tx.amount * RESOURCE_BACKING_RATIO {
            return Err("INSUFFICIENT_RESOURCE_BACKING");
        }
        tx.tx_id = self.tx_count as u64;
        tx.timestamp_ns = now_ns;
        tx.verified = true;
        tx.finalized = true;
        tx.reversal_prohibited = true;
        self.transactions[self.tx_count] = Some(tx);
        self.tx_count += 1;
        self.total_currency_in_circulation[tx.currency as usize] += tx.amount;
        self.update_audit_checksum();
        Ok(tx.tx_id)
    }
    pub fn distribute_basic_income(&mut self, now_ns: u64) -> Result<u64, &'static str> {
        let mut distributed = 0u64;
        for i in 0..self.account_count {
            if let Some(ref mut account) = self.accounts[i] {
                if account.basic_income_eligible && !account.frozen {
                    account.balances[0] += PHOENIX_BASIC_INCOME_USD_MONTH;
                    account.last_transaction_ns = now_ns;
                    distributed += 1;
                }
            }
        }
        self.basic_income_distributed_usd += distributed as f64 * PHOENIX_BASIC_INCOME_USD_MONTH;
        self.basic_income_recipients = distributed;
        self.last_distribution_ns = now_ns;
        self.update_audit_checksum();
        Ok(distributed)
    }
    pub fn compute_gini_coefficient(&mut self) -> f64 {
        if self.account_count == 0 { return 0.0; }
        let mut wealths = [0.0f64; MAX_ACCOUNTS];
        for i in 0..self.account_count {
            if let Some(ref account) = self.accounts[i] {
                wealths[i] = account.compute_total_wealth();
            }
        }
        wealths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let mut cumulative = 0.0;
        let mut total_wealth = 0.0;
        for i in 0..self.account_count {
            total_wealth += wealths[i];
        }
        if total_wealth == 0.0 { return 0.0; }
        for i in 0..self.account_count {
            cumulative += (2 * (i as f64 + 1.0) - self.account_count as f64 - 1.0) * wealths[i];
        }
        self.gini_coefficient = cumulative / (self.account_count as f64 * total_wealth);
        self.gini_coefficient
    }
    pub fn compute_economic_velocity(&self, period_days: u32) -> f64 {
        let period_ns = period_days as u64 * 86400000000000;
        let mut tx_volume = 0.0;
        let now_ns = SystemTime::now_ns();
        for i in 0..self.tx_count {
            if let Some(ref tx) = self.transactions[i] {
                if now_ns - tx.timestamp_ns < period_ns {
                    tx_volume += tx.amount;
                }
            }
        }
        let money_supply = self.total_currency_in_circulation.iter().sum::<f64>();
        if money_supply == 0.0 { return 0.0; }
        self.economic_velocity = tx_volume / money_supply;
        self.economic_velocity
    }
    pub fn get_system_status(&self, now_ns: u64) -> CurrencySystemStatus {
        let active_accounts = self.accounts.iter()
            .filter(|a| a.as_ref().map(|acc| !acc.frozen).unwrap_or(false))
            .count();
        CurrencySystemStatus {
            system_id: self.system_id,
            total_accounts: self.account_count,
            active_accounts,
            total_transactions: self.tx_count,
            total_currency_circulation: self.total_currency_in_circulation.iter().sum(),
            total_resource_value: self.total_resource_value,
            basic_income_distributed: self.basic_income_distributed_usd,
            basic_income_recipients: self.basic_income_recipients,
            gini_coefficient: self.gini_coefficient,
            economic_velocity: self.economic_velocity,
            inflation_rate_pct: self.inflation_rate_pct,
            last_distribution_ns: self.last_distribution_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.account_count as u64).wrapping_mul(self.tx_count as u64);
        sum ^= (self.basic_income_distributed_usd as u64);
        sum ^= self.basic_income_recipients;
        for i in 0..self.account_count {
            if let Some(ref account) = self.accounts[i] {
                sum ^= account.account_id.wrapping_mul(account.compute_total_wealth() as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.account_count as u64).wrapping_mul(self.tx_count as u64);
        sum ^= (self.basic_income_distributed_usd as u64);
        sum ^= self.basic_income_recipients;
        for i in 0..self.account_count {
            if let Some(ref account) = self.accounts[i] {
                sum ^= account.account_id.wrapping_mul(account.compute_total_wealth() as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct CurrencySystemStatus {
    pub system_id: u64,
    pub total_accounts: usize,
    pub active_accounts: usize,
    pub total_transactions: usize,
    pub total_currency_circulation: f64,
    pub total_resource_value: f64,
    pub basic_income_distributed: f64,
    pub basic_income_recipients: u64,
    pub gini_coefficient: f64,
    pub economic_velocity: f64,
    pub inflation_rate_pct: f64,
    pub last_distribution_ns: u64,
    pub last_update_ns: u64,
}

impl CurrencySystemStatus {
    pub fn economic_health_index(&self) -> f64 {
        let equality_score = 1.0 - self.gini_coefficient;
        let velocity_score = if self.economic_velocity > 0.5 && self.economic_velocity < 5.0 { 1.0 } else { 0.6 };
        let inclusion_score = self.active_accounts as f64 / self.total_accounts.max(1) as f64;
        let basic_income_coverage = self.basic_income_recipients as f64 / self.active_accounts.max(1) as f64;
        (equality_score * 0.3 + velocity_score * 0.25 + inclusion_score * 0.25 + basic_income_coverage * 0.2).min(1.0)
    }
}
