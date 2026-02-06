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
