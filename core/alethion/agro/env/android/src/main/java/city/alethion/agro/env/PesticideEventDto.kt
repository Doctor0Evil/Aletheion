package city.alethion.agro.env

data class GeoCellId(
    val township: Int,
    val range: Int,
    val section: Int,
    val quarter: Int,
)

enum class TribalScope {
    None,
    JointStewardship,
    IndependentStewardship,
}

data class FpicConsentTag(
    val tribalCode: String,
    val consentEpochUtc: Long,
    val consentVersion: Int,
    val onDeviceOnly: Boolean,
)

data class PesticideEventDto(
    val eventEpochUtc: Long,
    val geoCell: GeoCellId,
    val phoenixSoil: String,
    val activeIngredient: String,
    val productEpaRegNo: String,
    val applicationMethod: String,
    val rateValue: Int,
    val rateUnit: String,
    val totalAreaSqM: Int,
    val applicatorEpaEstablishmentId: String,
    val operatorLocalId: String,
    val tribalScope: TribalScope,
    val fpicConsent: FpicConsentTag?,
)

sealed class PesticideEventErrorDto {
    data object MissingRegNumber : PesticideEventErrorDto()
    data object InvalidRateUnit : PesticideEventErrorDto()
    data object ZeroArea : PesticideEventErrorDto()
    data object InvalidGeoCell : PesticideEventErrorDto()
    data object MissingApplicator : PesticideEventErrorDto()
    data object MissingFpicForTribalScope : PesticideEventErrorDto()
}

fun PesticideEventDto.validate(): PesticideEventErrorDto? {
    if (productEpaRegNo.isBlank()) {
        return PesticideEventErrorDto.MissingRegNumber
    }
    if (totalAreaSqM <= 0) {
        return PesticideEventErrorDto.ZeroArea
    }
    if (geoCell.section !in 1..36) {
        return PesticideEventErrorDto.InvalidGeoCell
    }
    if (applicatorEpaEstablishmentId.isBlank()) {
        return PesticideEventErrorDto.MissingApplicator
    }
    if (tribalScope != TribalScope.None) {
        val c = fpicConsent ?: return PesticideEventErrorDto.MissingFpicForTribalScope
        if (c.tribalCode.isBlank() || c.consentEpochUtc <= 0L) {
            return PesticideEventErrorDto.MissingFpicForTribalScope
        }
    }
    return null
}
