pub fn opt_string_trim_non_empty(v: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(s) = v {
        if s.trim().is_empty() {
            return Err(garde::Error::new("不能为空"));
        }
    }
    Ok(())
}

pub fn string_trim_min_len_8(v: &str, _ctx: &()) -> garde::Result {
    if v.trim().len() < 8 {
        return Err(garde::Error::new("长度不能小于 8"));
    }
    Ok(())
}

pub fn opt_u64_min_10(v: &Option<u64>, _ctx: &()) -> garde::Result {
    if let Some(n) = v {
        if *n < 10 {
            return Err(garde::Error::new("不能小于 10"));
        }
    }
    Ok(())
}

pub fn string_basic_email(v: &str, _ctx: &()) -> garde::Result {
    let v = v.trim();
    if v.is_empty() {
        return Err(garde::Error::new("不能为空"));
    }
    let at_index = v.find('@');
    let Some(at_index) = at_index else {
        return Err(garde::Error::new("邮箱格式不合法"));
    };
    if at_index == 0 || at_index + 1 >= v.len() {
        return Err(garde::Error::new("邮箱格式不合法"));
    }
    if !v[at_index + 1..].contains('.') {
        return Err(garde::Error::new("邮箱格式不合法"));
    }
    Ok(())
}

pub fn opt_string_basic_email(v: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(s) = v {
        string_basic_email(s, &())?;
    }
    Ok(())
}

pub fn opt_username_format(v: &Option<String>, _ctx: &()) -> garde::Result {
    let Some(username) = v else {
        return Ok(());
    };

    let username = username.trim();
    if username.contains('@') {
        return Err(garde::Error::new("用户名不能包含 @"));
    }

    if !username.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err(garde::Error::new("用户名必须包含字母"));
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(garde::Error::new("用户名只能包含字母、数字或下划线"));
    }

    Ok(())
}
