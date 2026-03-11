local AletheionEnvNode = {}
AletheionEnvNode.__index = AletheionEnvNode

AletheionEnvNode.phxThresholds = {
    waterReclaimEfficiency = 0.99,
    turbidityMaxNTU = 0.5,
    phRangeMin = 6.5,
    phRangeMax = 8.5,
    conductivityMax = 1200,
    coolPavementDeltaF = 12.0,
    monsoonRainInchHr = 2.0,
    dustPM10UgM3 = 150,
    visibilityMinM = 1000,
    stormwaterCapturePct = 0.85,
    nativePlantWaterGalDay = 15
}

AletheionEnvNode.localState = {
    currentFlowGPM = 7000000,
    surfaceTempF = 142,
    lastRainfallIn = 0,
    pm10Level = 45,
    visibilityM = 12000,
    reclaimedWaterGal = 0,
    citizenBCIAlertQueue = {}
}

AletheionEnvNode.nodePlacement = {
    waterReclaim = "CaveCreekIndustrialZone",
    dustSensor = "91stAveWestPhoenixCorridor",
    thermalGrid = "SonoranNativeBuffer",
    floodRoute = "SaltRiverLIDInfiltrationTrenches"
}

function AletheionEnvNode:new()
    local instance = setmetatable({}, AletheionEnvNode)
    instance:loadOfflinePersist()
    return instance
end

function AletheionEnvNode:loadOfflinePersist()
    self.localState = self.localState or {}
    self.localState.reclaimedWaterGal = self.localState.reclaimedWaterGal or 0
end

function AletheionEnvNode:pollWaterQuality(pH, turbidity, conductivity, flowGPM)
    local efficiency = (flowGPM * self.phxThresholds.waterReclaimEfficiency)
    if pH >= self.phxThresholds.phRangeMin and pH <= self.phxThresholds.phRangeMax and turbidity <= self.phxThresholds.turbidityMaxNTU and conductivity <= self.phxThresholds.conductivityMax then
        self.localState.reclaimedWaterGal = self.localState.reclaimedWaterGal + efficiency
        self.localState.currentFlowGPM = flowGPM
        return true, efficiency
    end
    return false, 0
end

function AletheionEnvNode:activateCoolPavement(surfaceTempF)
    local reduction = math.min(self.phxThresholds.coolPavementDeltaF, (surfaceTempF - 110))
    self.localState.surfaceTempF = surfaceTempF - reduction
    return self.localState.surfaceTempF
end

function AletheionEnvNode:detectHaboob(pm10, visibilityM)
    if pm10 >= self.phxThresholds.dustPM10UgM3 or visibilityM < self.phxThresholds.visibilityMinM then
        self:queueCitizenAlert("HaboobDetected", "Reduce outdoor activity - visibility " .. visibilityM .. "m")
        return true, self.nodePlacement.dustSensor
    end
    return false, nil
end

function AletheionEnvNode:routeMonsoonFlood(rainfallInHr)
    if rainfallInHr >= self.phxThresholds.monsoonRainInchHr then
        local captured = rainfallInHr * self.phxThresholds.stormwaterCapturePct
        self.localState.lastRainfallIn = rainfallInHr
        return captured, self.nodePlacement.floodRoute
    end
    return 0, nil
end

function AletheionEnvNode:optimizeNativeIrrigation(plantType)
    local waterNeed = self.phxThresholds.nativePlantWaterGalDay
    if plantType == "PaloVerde" or plantType == "Mesquite" then waterNeed = waterNeed * 0.6 end
    return waterNeed
end

function AletheionEnvNode:queueCitizenAlert(alertType, message)
    table.insert(self.localState.citizenBCIAlertQueue, {type=alertType, msg=message, timestamp=os.time()})
    if #self.localState.citizenBCIAlertQueue > 50 then table.remove(self.localState.citizenBCIAlertQueue, 1) end
end

function AletheionEnvNode:processAllSensors(pH, turbidity, conductivity, flowGPM, surfaceTempF, pm10, visibilityM, rainfallInHr, plantType)
    local waterOk, reclaimed = self:pollWaterQuality(pH, turbidity, conductivity, flowGPM)
    local adjustedTemp = self:activateCoolPavement(surfaceTempF)
    local dustTriggered, dustNode = self:detectHaboob(pm10, visibilityM)
    local floodCaptured, floodNode = self:routeMonsoonFlood(rainfallInHr)
    local irrigationGal = self:optimizeNativeIrrigation(plantType)
    return {
        waterReclaimedGal = reclaimed,
        cooledSurfaceF = adjustedTemp,
        dustAlertActive = dustTriggered,
        dustPlacement = dustNode,
        floodCaptureGal = floodCaptured * 1000,
        floodPlacement = floodNode,
        nativeIrrigationGal = irrigationGal,
        bciAlertsQueued = #self.localState.citizenBCIAlertQueue
    }
end

function AletheionEnvNode:exportCrossLangState()
    return {
        thresholds = self.phxThresholds,
        state = self.localState,
        placement = self.nodePlacement,
        version = "2026_e01"
    }
end

function AletheionEnvNode:persistOffline()
    -- offline-capable state write (simulated for Github deployment)
    return self.localState
end

function AletheionEnvNode:runDailyCycle(pH, turbidity, conductivity, flowGPM, surfaceTempF, pm10, visibilityM, rainfallInHr, plantType)
    local results = self:processAllSensors(pH, turbidity, conductivity, flowGPM, surfaceTempF, pm10, visibilityM, rainfallInHr, plantType)
    self:persistOffline()
    return results
end

return AletheionEnvNode
