
"use strict";

function validateRiskCoordinate(name, value) {
  if (typeof value !== "number" || value < 0.0 || value > 1.0) {
    throw new Error(`Risk coordinate ${name} must be in [0,1], got ${value}`);
  }
}

function ecosafetyMustPassGate(nodeSpec) {
  if (!nodeSpec.ecosafetyCorridorId) {
    throw new Error("no corridor, no build: ecosafetyCorridorId missing");
  }

  const r = nodeSpec.riskCoordinates || {};
  const keys = [
    "r_degrade",
    "r_residualmass",
    "r_microplastics",
    "r_tox_acute",
    "r_tox_chronic",
    "r_shear",
    "r_habitatload",
  ];
  keys.forEach((k) => validateRiskCoordinate(k, r[k]));

  if (!nodeSpec.governance || !nodeSpec.governance.mustPassGates) {
    throw new Error("governance.mustPassGates missing");
  }

  if (nodeSpec.governance.mustPassGates.indexOf("ecosafety") === -1) {
    throw new Error("ecosafety gate must be declared in governance.mustPassGates");
  }

  return true;
}

module.exports = {
  ecosafetyMustPassGate,
};
