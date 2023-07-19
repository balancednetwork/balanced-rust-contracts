use cosmwasm_std::Addr;

pub struct AssetManagerContract(Addr);

impl AssetManagerContract {
    pub fn addr(&self) -> &Addr {
        &self.0;
    }
    
}