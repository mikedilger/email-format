
use super::types::{AddressList, Address, Mailbox, Group, NameAddr, AddrSpec,
                   GroupList, MailboxList};

/// This type represents an Email Address in a way that is simpler and more
/// directly useful than the ABNF-based rfc5322 types. It is not used by the
/// main parser, but may be useful to consumers of this library.
pub struct EmailAddress {
    pub display_name: Option<String>,
    pub local_part: String,
    pub domain: String,
}

impl EmailAddress {
    pub fn from_addresses(addr: &AddressList) -> Vec<EmailAddress>
    {
        let mut output: Vec<EmailAddress> = Vec::new();
        for address in &addr.0 {
            output.extend( EmailAddress::from_address(address).into_iter() );
        }
        output
    }

    pub fn from_address(addr: &Address) -> Vec<EmailAddress>
    {
        match *addr {
            Address::Mailbox(ref mbox) => vec![ EmailAddress::from_mailbox(mbox) ],
            Address::Group(ref group) => EmailAddress::from_group(group),
        }
    }

    pub fn from_mailbox(mbox: &Mailbox) -> EmailAddress
    {
        match *mbox {
            Mailbox::NameAddr(ref name_addr) => EmailAddress::from_name_addr(name_addr),
            Mailbox::AddrSpec(ref addr_spec) => EmailAddress::from_addr_spec(addr_spec),
        }
    }

    pub fn from_name_addr(name_addr: &NameAddr) -> EmailAddress
    {
        let mut email_address = EmailAddress::from_addr_spec(
            &name_addr.angle_addr.addr_spec);
        if let Some(ref display_name) = name_addr.display_name {
            email_address.display_name = Some(format!("{}", display_name));
        }
        email_address
    }

    pub fn from_addr_spec(addr_spec: &AddrSpec) -> EmailAddress
    {
        EmailAddress {
            display_name: None,
            local_part: format!("{}", addr_spec.local_part),
            domain: format!("{}", addr_spec.domain),
        }
    }

    pub fn from_group(group: &Group) -> Vec<EmailAddress>
    {
        match group.group_list {
            Some(ref gl) => EmailAddress::from_group_list(gl),
            None => Vec::new(),
        }
    }

    pub fn from_group_list(group_list: &GroupList) -> Vec<EmailAddress>
    {
        match *group_list {
            GroupList::MailboxList(ref mbl) => EmailAddress::from_mailbox_list(mbl),
            _ => Vec::new(),
        }
    }

    pub fn from_mailbox_list(mbl: &MailboxList) -> Vec<EmailAddress>
    {
        let mut output: Vec<EmailAddress> = Vec::new();
        for mailbox in &mbl.0 {
            output.push( EmailAddress::from_mailbox(mailbox) );
        }
        output
    }
}
