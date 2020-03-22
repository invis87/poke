#[derive(Debug, Fail)]
pub enum ConnectionToolsError {
    #[fail(display = "fail to get sockets info: {}", message)]
    FailToGetSocketsInfo { message: String },
}

#[cfg(test)]
mod test {
    use super::ConnectionToolsError;

    #[test]
    fn string_representation() {
        let sockets_info_failure = ConnectionToolsError::FailToGetSocketsInfo {
            message: "xxxx".to_owned(),
        };
        let failure = failure::Error::from(sockets_info_failure);

        let result_string = format!("{}", failure);
        assert_eq!(result_string, "fail to get sockets info: xxxx");
    }
}
