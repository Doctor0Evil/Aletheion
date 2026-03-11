export class PesticideEventError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

export function validatePesticideEvent(ev) {
  if (!ev.productEpaRegNo || ev.productEpaRegNo.trim() === "") {
    throw new PesticideEventError("MissingRegNumber", "EPA registration number is required.");
  }
  if (!ev.totalAreaSqM || ev.totalAreaSqM <= 0) {
    throw new PesticideEventError("ZeroArea", "Total treated area must be greater than zero.");
  }
  if (!ev.section || ev.section < 1 || ev.section > 36) {
    throw new PesticideEventError("InvalidGeoCell", "Section must be between 1 and 36.");
  }
  if (!ev.applicatorEpaEstablishmentId || ev.applicatorEpaEstablishmentId.trim() === "") {
    throw new PesticideEventError("MissingApplicator", "Applicator EPA establishment id is required.");
  }
  if (ev.tribalScope && ev.tribalScope !== "None") {
    const c = ev.fpicConsent;
    if (!c || !c.tribalCode || c.tribalCode.trim() === "" || !c.consentEpochUtc || c.consentEpochUtc <= 0) {
      throw new PesticideEventError("MissingFpicForTribalScope", "FPIC consent metadata is required for tribal scope.");
    }
  }
  return true;
}
