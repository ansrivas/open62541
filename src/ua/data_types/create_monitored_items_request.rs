use crate::ua;

crate::data_type!(
    CreateMonitoredItemsRequest,
    UA_CreateMonitoredItemsRequest,
    UA_TYPES_CREATEMONITOREDITEMSREQUEST
);

impl CreateMonitoredItemsRequest {
    #[must_use]
    pub fn with_subscription_id(mut self, subscription_id: &ua::SubscriptionId) -> Self {
        self.0.subscriptionId = subscription_id.into_inner();
        self
    }

    #[must_use]
    pub fn with_items_to_create(
        mut self,
        items_to_create: &[ua::MonitoredItemCreateRequest],
    ) -> Self {
        let array = ua::Array::from_slice(items_to_create);
        array.move_into(&mut self.0.itemsToCreateSize, &mut self.0.itemsToCreate);
        self
    }
}