use cosmwasm_std::Addr;

pub struct AssetManagerContract(Addr);

impl AssetManagerContract {
    pub fn addr(&self) -> &Addr {
        &self.0;
    }
    

    pub fn store_code() -> u64 {
        let contract = ContractWrapper::new(execute,instantiate);
    }


}