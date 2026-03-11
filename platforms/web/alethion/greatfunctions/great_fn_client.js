export class GreatFnId {
  constructor(city, district, block, lane) {
    this.city = city;
    this.district = district;
    this.block = block;
    this.lane = lane;
  }
  toString() {
    return `${this.city}-${this.district}-${this.block}-${this.lane}`;
  }
}

export class GreatFnCall {
  constructor(target, cityZone, epoch, payload = {}, priority = 0) {
    this.target = target;
    this.cityZone = cityZone;
    this.epoch = epoch;
    this.payload = payload;
    this.priority = priority;
  }
}

export class GreatFnClient {
  constructor(transport) {
    this.transport = transport;
  }

  route(call) {
    const body = {
      target: call.target.toString(),
      zone: call.cityZone,
      epoch: call.epoch,
      payload: call.payload,
      priority: call.priority,
    };
    return this.transport.post("/great_fn/route", body);
  }

  tick() {
    return this.transport.post("/great_fn/tick", {});
  }
}
