import { computed, type Ref } from "vue";
import type { FeeConfig, FlattenedMaterial } from "../types/crafting";

export interface MaterialPricing {
  key: string;
  item_id: number | null;
  item_name: string;
  quantity_needed: number;
  customer_provides: number;
  you_provide: number;
  unit_price: number | null;
  price_source: "market" | "craft" | "vendor" | null;
  your_cost: number | null;
  their_value: number | null;
}

export interface PriceCalculation {
  materials: MaterialPricing[];
  yourMaterialCost: number;
  theirMaterialValue: number;
  totalMaterialCost: number;
  totalCrafts: number;
  perCraftTotal: number;
  materialPctFee: number;
  flatFee: number;
  totalFee: number;
  chargeCustomer: number;
  hasUnknownPrices: boolean;
}

export function usePriceCalculator(
  flatMaterials: Ref<Map<string, FlattenedMaterial>>,
  materialPrices: Ref<Map<string, { unitPrice: number | null; source: "market" | "craft" | "vendor" | null }>>,
  customerProvides: Ref<Record<string, number>>,
  feeConfig: Ref<FeeConfig>,
  totalCrafts: Ref<number>,
) {
  const calculation = computed<PriceCalculation>(() => {
    const materials: MaterialPricing[] = [];
    let yourMaterialCost = 0;
    let theirMaterialValue = 0;
    let hasUnknownPrices = false;

    for (const [key, mat] of flatMaterials.value) {
      const priceInfo = materialPrices.value.get(key);
      const unitPrice = priceInfo?.unitPrice ?? null;
      const source = priceInfo?.source ?? null;
      const custQty = Math.min(customerProvides.value[key] ?? 0, mat.expected_quantity);
      const youProvide = Math.max(0, mat.expected_quantity - custQty);

      const yourCost = unitPrice !== null ? Math.round(unitPrice * youProvide) : null;
      const theirValue = unitPrice !== null ? Math.round(unitPrice * custQty) : null;

      if (unitPrice === null) hasUnknownPrices = true;
      if (yourCost !== null) yourMaterialCost += yourCost;
      if (theirValue !== null) theirMaterialValue += theirValue;

      materials.push({
        key,
        item_id: mat.item_id,
        item_name: mat.item_name,
        quantity_needed: mat.expected_quantity,
        customer_provides: custQty,
        you_provide: youProvide,
        unit_price: unitPrice,
        price_source: source,
        your_cost: yourCost,
        their_value: theirValue,
      });
    }

    const totalMaterialCost = yourMaterialCost + theirMaterialValue;
    const crafts = totalCrafts.value;
    const fee = feeConfig.value;

    const perCraftTotal = fee.per_craft_fee * crafts;

    let materialPctFee = 0;
    const pctBase =
      fee.material_pct_basis === "yours"
        ? yourMaterialCost
        : fee.material_pct_basis === "theirs"
          ? theirMaterialValue
          : totalMaterialCost;
    materialPctFee = Math.round((fee.material_pct / 100) * pctBase);

    const flatFee = fee.flat_fee;
    const totalFeeAmount = perCraftTotal + materialPctFee + flatFee;
    const chargeCustomer = yourMaterialCost + totalFeeAmount;

    return {
      materials,
      yourMaterialCost,
      theirMaterialValue: theirMaterialValue,
      totalMaterialCost,
      totalCrafts: crafts,
      perCraftTotal,
      materialPctFee,
      flatFee,
      totalFee: totalFeeAmount,
      chargeCustomer,
      hasUnknownPrices,
    };
  });

  return { calculation };
}
