local M = {}

local function is_zero_str(s)
    return s == nil or s == ""
end

local function has_fpic(scope, consent)
    if scope == "None" then
        return true
    end
    if consent == nil then
        return false
    end
    if not consent.tribal_code or consent.tribal_code == "" then
        return false
    end
    if not consent.consent_epoch_utc or consent.consent_epoch_utc <= 0 then
        return false
    end
    return true
end

function M.validate(event)
    if is_zero_str(event.product_epa_reg_no) then
        return false, "MissingRegNumber"
    end
    if not event.total_area_sq_m or event.total_area_sq_m == 0 then
        return false, "ZeroArea"
    end
    if not event.section or event.section < 1 or event.section > 36 then
        return false, "InvalidGeoCell"
    end
    if is_zero_str(event.applicator_epa_establishment_id) then
        return false, "MissingApplicator"
    end
    if not has_fpic(event.tribal_scope, event.fpic_consent) then
        return false, "MissingFpicForTribalScope"
    end
    return true, "Ok"
end

return M
